//! UniFFI 境界（docs/08 8.5）。純粋コア qrac-core ＋ I/O層 qrac-render を Swift/Kotlin に公開する。
//! ここは「橋渡し」だけ。決定論ロジックは一切持たない。

use qrac_core::derive_from_string;
use qrac_core::hash::make_seed;
use qrac_core::normalize::normalize_key;
use qrac_render::{render_png, ArtifactDb, CANVAS_H, CANVAS_W};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

uniffi::setup_scaffolding!();

/// 遺物DB（メモリ・初回シード）。Step 3 で実DBファイルに差し替える。
fn db() -> &'static Mutex<ArtifactDb> {
    static DB: OnceLock<Mutex<ArtifactDb>> = OnceLock::new();
    DB.get_or_init(|| {
        let db = ArtifactDb::open_in_memory().expect("open db");
        db.seed_full().expect("seed db");
        db.seed_references().expect("seed references"); // 出土の系譜（提案01）
        Mutex::new(db)
    })
}

/// ベース画像アセットの所在（qrac-assetgen の出力 `<dist>/assets`）。未設定なら手続き生成。
fn assets_dir() -> &'static Mutex<Option<PathBuf>> {
    static D: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    D.get_or_init(|| Mutex::new(None))
}

/// アセットディレクトリを設定（例: アプリ同梱の assets/ パス）。
#[uniffi::export]
pub fn configure_assets(dir: String) {
    *assets_dir().lock().unwrap() = Some(PathBuf::from(dir));
}

/// 色違い HSV 補正。
#[derive(uniffi::Record)]
pub struct ColorMod {
    pub h_shift: f64,
    pub s_mul: f64,
    pub v_mul: f64,
}

/// 破損状態。
#[derive(uniffi::Record)]
pub struct Damage {
    pub chip: bool,
    pub crack: bool,
    pub wear: bool,
}

/// 図鑑表示に必要な遺物属性（DB選択・画像合成は次フェーズ）。
#[derive(uniffi::Record)]
pub struct Artifact {
    pub artifact_hash: String,
    pub base_rarity: u32,
    pub era_bonus: u32,
    pub final_rarity: u32,
    pub is_mythic: bool,
    pub civ: String,
    pub era: String,
    pub category: String,
    /// 型キー（=image_set_id, 提案01 判断D）。収集の imageSetId バックフィル・参照照合用。
    pub image_set_id: i64,
    pub color: ColorMod,
    pub dirt_layer_id: String,
    pub damage: Damage,
    pub preservation_score: f64,
}

fn to_artifact(a: qrac_core::DerivedAttributes, image_set_id: i64) -> Artifact {
    Artifact {
        artifact_hash: a.artifact_hash,
        base_rarity: a.base_rarity,
        era_bonus: a.era_bonus,
        final_rarity: a.final_rarity,
        is_mythic: a.is_mythic,
        civ: a.civ,
        era: a.era,
        category: a.category,
        image_set_id,
        color: ColorMod {
            h_shift: a.color_mod.h_shift,
            s_mul: a.color_mod.s_mul,
            v_mul: a.color_mod.v_mul,
        },
        dirt_layer_id: a.dirt_layer_id,
        damage: Damage {
            chip: a.damage.chip,
            crack: a.damage.crack,
            wear: a.damage.wear,
        },
        preservation_score: a.preservation_score,
    }
}

/// text + 属性から型キー（image_set_id）を解決（DB選択, バックフィル兼用）。
fn image_set_of(text: &str, attr: &qrac_core::DerivedAttributes) -> i64 {
    let seed = make_seed(&normalize_key(text.as_bytes()));
    let db = db().lock().unwrap();
    db.select_base(&seed, &attr.civ, &attr.era, &attr.category, attr.base_rarity)
        .image_set_id
}

/// QR文字列 → 遺物（年は内容から自動抽出）。
#[uniffi::export]
pub fn derive_qr(text: String) -> Artifact {
    let attr = derive_from_string(&text, None);
    let set = image_set_of(&text, &attr);
    to_artifact(attr, set)
}

/// QR文字列 → 遺物（年を明示。None=年代取得不可として +0）。
#[uniffi::export]
pub fn derive_qr_with_year(text: String, year: Option<i32>) -> Artifact {
    // Some(None) = 強制 null。ここでは year を「指定 or 取得不可」として渡す。
    let attr = derive_from_string(&text, Some(year));
    let set = image_set_of(&text, &attr);
    to_artifact(attr, set)
}

/// 合成済み画像（PNGバイト列＋寸法）＋解説文。
#[derive(uniffi::Record)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
    /// 選ばれたベース遺物の名称（DB由来）。
    pub base_name: String,
    /// 型キー（=image_set_id, 提案01 判断D）。
    pub image_set_id: i64,
    /// 当たったフォールバック段（1..6, 7=GLOBAL）。
    pub matched_stage: u8,
    /// 民明書房調の解説文（「詳説 世界の遺物」より抜粋の体裁。参照文を含みうる）。
    pub description: String,
}

/// 関連遺物（出土の系譜, 提案01 §5）。詳細画面「関連遺物」セクション用。
#[derive(uniffi::Record)]
pub struct Reference {
    /// 参照先の型キー（image_set_id）。
    pub to_set: i64,
    /// 'pair' | 'cites' | 'rival'。
    pub kind: String,
    /// 参照先の型呼称（§3.4・詳細チップと同語彙）。
    pub to_label: String,
}

/// 解説文の言語。表示テキストのみに影響し、決定論（hash/属性）には無関係。
#[derive(uniffi::Enum)]
pub enum Lang {
    Ja,
    En,
}

impl From<Lang> for qrac_core::Lang {
    fn from(l: Lang) -> Self {
        match l {
            Lang::Ja => qrac_core::Lang::Ja,
            Lang::En => qrac_core::Lang::En,
        }
    }
}

/// 型キー → 解決済み参照メタ（判断B: DBアクセスは flavor の外で行い、引数で渡す）。
fn resolve_refs(
    db: &ArtifactDb,
    image_set_id: i64,
    lang: qrac_core::Lang,
) -> Vec<qrac_core::RefMeta> {
    db.references_from(image_set_id)
        .into_iter()
        .filter_map(|(to_set, kind)| {
            qrac_core::RefKind::parse(&kind).map(|k| qrac_core::RefMeta {
                kind: k,
                to_set,
                to_label: qrac_core::type_label_from_set(to_set, lang),
            })
        })
        .collect()
}

fn render_impl(text: &str, attr: &qrac_core::DerivedAttributes, lang: Lang) -> RenderedImage {
    let seed = make_seed(&normalize_key(text.as_bytes()));
    let core_lang: qrac_core::Lang = lang.into();
    let (base, refs) = {
        let db = db().lock().unwrap();
        let base = db.select_base(
            &seed,
            &attr.civ,
            &attr.era,
            &attr.category,
            attr.base_rarity,
        );
        let refs = resolve_refs(&db, base.image_set_id, core_lang);
        (base, refs)
    };
    // アセットディレクトリ配下のレイヤーPNGで6層合成（無いレイヤーは手続き生成にフォールバック）。
    let png = {
        let dir = assets_dir().lock().unwrap();
        render_png(attr, &base, dir.as_deref())
    };
    let description = qrac_core::flavor::describe(&seed, attr, &refs, core_lang);
    RenderedImage {
        width: CANVAS_W,
        height: CANVAS_H,
        png,
        base_name: base.name,
        image_set_id: base.image_set_id,
        matched_stage: base.matched_stage,
        description,
    }
}

/// QR文字列 → 合成画像（derive → DB選択 → 6層合成 → PNG）。lang は解説文の言語。
#[uniffi::export]
pub fn render_qr(text: String, lang: Lang) -> RenderedImage {
    let attr = derive_from_string(&text, None);
    render_impl(&text, &attr, lang)
}

/// 年指定つきレンダ（デバッグ/バランス確認用。None=年代取得不可として +0）。
#[uniffi::export]
pub fn render_qr_with_year(text: String, year: Option<i32>, lang: Lang) -> RenderedImage {
    let attr = derive_from_string(&text, Some(year));
    render_impl(&text, &attr, lang)
}

/// 解説文のみを生成（画像なし・軽量）。言語切替時の再生成や展示室表示に使う。
#[uniffi::export]
pub fn describe_qr(text: String, lang: Lang) -> String {
    let attr = derive_from_string(&text, None);
    let seed = make_seed(&normalize_key(text.as_bytes()));
    let core_lang: qrac_core::Lang = lang.into();
    let refs = {
        let db = db().lock().unwrap();
        let set = db
            .select_base(&seed, &attr.civ, &attr.era, &attr.category, attr.base_rarity)
            .image_set_id;
        resolve_refs(&db, set, core_lang)
    };
    qrac_core::flavor::describe(&seed, &attr, &refs, core_lang)
}

// ── 出土の系譜（提案01）の FFI 表面 ─────────────────────────────────────────

/// 型呼称（判断C・Q7=(i)）。Swift はこれを呼んで参照先名／チップを Rust 語彙で表示する。
#[uniffi::export]
pub fn type_label(civ: String, era: String, category: String, lang: Lang) -> String {
    qrac_core::type_label(&civ, &era, &category, lang.into())
}

/// 軸単体の表示呼称（Q7=(i): Swift の civName/eraName/categoryName が呼ぶ単一の出所）。
#[uniffi::export]
pub fn civ_label(civ: String, lang: Lang) -> String {
    qrac_core::civ_label(&civ, lang.into()).to_string()
}

#[uniffi::export]
pub fn era_label(era: String, lang: Lang) -> String {
    qrac_core::era_label(&era, lang.into()).to_string()
}

#[uniffi::export]
pub fn category_label(category: String, lang: Lang) -> String {
    qrac_core::category_label(&category, lang.into()).to_string()
}

/// image_set_id から型呼称を解決（GLOBAL/範囲外は汎称）。
#[uniffi::export]
pub fn type_label_for_set(image_set_id: i64, lang: Lang) -> String {
    qrac_core::type_label_from_set(image_set_id, lang.into())
}

/// 型 `image_set_id` の関連遺物一覧（詳細画面「関連遺物」用, §5）。
#[uniffi::export]
pub fn references_of(image_set_id: i64, lang: Lang) -> Vec<Reference> {
    let core_lang: qrac_core::Lang = lang.into();
    let db = db().lock().unwrap();
    db.references_from(image_set_id)
        .into_iter()
        .map(|(to_set, kind)| Reference {
            to_set,
            to_label: qrac_core::type_label_from_set(to_set, core_lang),
            kind,
        })
        .collect()
}

/// 合本解説（型ペア・canonical, 判断E）。両型所持時に詳細画面で表示。
/// kind 不正値は 'pair' とみなす。
#[uniffi::export]
pub fn describe_pair(set_a: i64, set_b: i64, kind: String, lang: Lang) -> String {
    let k = qrac_core::RefKind::parse(&kind).unwrap_or(qrac_core::RefKind::Pair);
    qrac_core::describe_pair(set_a, set_b, k, lang.into())
}
