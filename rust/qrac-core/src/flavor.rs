//! 遺物の解説文（民明書房調）。docs/04 4.5 の個体差テキストの一種。
//! 「詳説 世界の遺物（萬象書房 1890年刊）」からの抜粋という体裁。決定論（seed 由来）。
//! 日本語/英語の両対応。言語は表示テキストのみに影響し、ハッシュ・属性には一切影響しない。
//! 各言語の語句配列は同じ長さに保ち、同一 seed で「同じ選択」を別言語で出す。
use crate::genealogy::{RefKind, RefMeta};
use crate::hash::uint_below;
use crate::types::DerivedAttributes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ja,
    En,
}

fn pick<'a>(seed: &[u8], tag: &str, arr: &[&'a str]) -> &'a str {
    arr[uint_below(seed, tag, arr.len() as u32) as usize]
}

/// 架空の発掘地名（決定論）。同一 seed なら日英で同じ綴り構造。
fn gen_place(seed: &[u8], lang: Lang) -> String {
    const SYL_JA: [&str; 14] = [
        "ア", "ル", "サ", "ネブ", "カ", "トゥ", "メ", "ゾ", "ラ", "ウ", "ヴ", "ガ", "シ", "ロ",
    ];
    const SYL_EN: [&str; 14] = [
        "A", "ru", "sa", "neb", "ka", "tu", "me", "zo", "ra", "u", "va", "ga", "shi", "ro",
    ];
    const SUF_JA: [&str; 6] = ["盆地", "遺跡", "谷", "高原", "砂海", "氷河"];
    const SUF_EN: [&str; 6] = [
        "Basin",
        "Ruins",
        "Valley",
        "Plateau",
        "Sea of Sand",
        "Glacier",
    ];

    let (syl, suf_tbl) = match lang {
        Lang::Ja => (SYL_JA, SUF_JA),
        Lang::En => (SYL_EN, SUF_EN),
    };
    let n = 2 + uint_below(seed, "text:place.len", 2); // 2..3 音節
    let mut name = String::new();
    for i in 0..n {
        let idx = uint_below(seed, &format!("text:place.{i}"), syl.len() as u32) as usize;
        name.push_str(syl[idx]);
    }
    let suf = suf_tbl[uint_below(seed, "text:place.suf", suf_tbl.len() as u32) as usize];
    match lang {
        Lang::Ja => format!("“{name}{suf}”"),
        Lang::En => {
            // 頭文字を大文字化（固有名詞らしく）
            let mut chars = name.chars();
            let head = chars
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();
            format!("“{head}{} {suf}”", chars.as_str())
        }
    }
}

/// 民明書房調の解説文を生成する（本文のみ。書名・出版社はUI側で付与）。
/// `refs` は呼び出し側（qrac-render/ffi）が解決した参照メタ（判断B: DBアクセスは外）。
/// 空なら従来どおり参照文なし。1件以上なら `seed + kind` で1件を決定論選択し本文末尾に1文付加。
pub fn describe(seed: &[u8], a: &DerivedAttributes, refs: &[RefMeta], lang: Lang) -> String {
    let body = match lang {
        Lang::Ja => describe_ja(seed, a),
        Lang::En => describe_en(seed, a),
    };
    match ref_sentence(seed, refs, lang) {
        Some(s) => format!("{body}{s}"),
        None => body,
    }
}

/// 参照文（1文）。refs から `seed:ref` で1件を決定論選択し、kind 別テンプレで描く。
/// EN は本文同様、先頭に半角スペースを置いて連結する。
fn ref_sentence(seed: &[u8], refs: &[RefMeta], lang: Lang) -> Option<String> {
    if refs.is_empty() {
        return None;
    }
    let idx = uint_below(seed, "text:ref", refs.len() as u32) as usize;
    let r = &refs[idx];
    let label = &r.to_label;
    Some(match (lang, r.kind) {
        (Lang::Ja, RefKind::Pair) => format!("なお本品は、〈{label}〉と対をなすものと伝わる。"),
        (Lang::Ja, RefKind::Cites) => {
            format!("その銘の片隅には、〈{label}〉への言及が確かに見て取れる。")
        }
        (Lang::Ja, RefKind::Rival) => {
            format!("また〈{label}〉とは、久しく覇を競いし好敵手であったという。")
        }
        (Lang::En, RefKind::Pair) => {
            format!(" It is, moreover, said to form a pair with the 〈{label}〉.")
        }
        (Lang::En, RefKind::Cites) => {
            format!(" In a corner of its inscription, a clear mention of the 〈{label}〉 may be discerned.")
        }
        (Lang::En, RefKind::Rival) => {
            format!(" It is told, too, to have long vied for supremacy with the 〈{label}〉, a worthy foe.")
        }
    })
}

fn describe_ja(seed: &[u8], a: &DerivedAttributes) -> String {
    let civ = match a.civ.as_str() {
        "desert" => "灼熱の砂漠に栄えし砂上の王朝",
        "ocean" => "大洋の底に沈みし海洋文明",
        "mountain" => "万年雪の霊峰に隠れ住みし山岳の民",
        "machine" => "歯車仕掛けの神を奉りし古代機械文明",
        "organic" => "生命と渾然一体たりし有機文明",
        _ => "いずことも知れぬ忘れられし民",
    };
    let era = match a.era.as_str() {
        "ancient" => "太古",
        "medieval" => "中世",
        "early_modern" => "近世",
        "modern" => "近代",
        "future" => "未だ来たらざる未来",
        _ => "悠久の刻",
    };
    let cat = match a.category.as_str() {
        "weapon" => "神聖なる武具",
        "ritual" => "厳粛なる祭祀の具",
        "daily" => "日々の暮らしを支えし什器",
        "architecture" => "壮麗なる殿堂の一片",
        "inscription" => "秘奥の文を刻みし碑",
        _ => "精緻を極めし機巧の部品",
    };
    let place = gen_place(seed, Lang::Ja);
    let opening = format!("{place}より出土せし本品は、{civ}に興りし、{era}の{cat}に他ならぬ。");
    let etym = pick(
        seed,
        "text:desc.etym",
        &[
            "今日、万人が時を計るに用うる暦法の根本が、実にこの文様より生じたことは、もはや論を俟たぬ。",
            "かの古の大賢が天地創成の理を悟りしは、これを一目せし刹那であったと、古文書は確かに伝えている。",
            "東西の交易路を遥々と渡り、遠き異邦の王侯すらも虜にしたというのだから、その価値は推して知るべしである。",
            "我々が日々何気なく口にする数多の言の葉、その語源のことごとくが本品に発すると説く碩学も、決して少なくはない。",
            "世界各地に散らばる『失われし宝』の伝説、その真の出所がまさしくこれであるとは、知る人ぞ知る秘事である。",
            "後世これを模したる贋作が無数に作られたが、いずれも本品の神韻には遠く及ばなんだと、記録は一様に嘆いている。",
        ],
    );
    let surface = surface_ja(a);
    let closing = pick(
        seed,
        "text:desc.close",
        &[
            "——この事実を知る者、今や天が下にただ一人とてあるまい。",
            "……諸君、これこそが歴史の闇に永く葬られし真実である。",
            "本書房が多年の研鑽の末、ここに白日の下へと晒すものなり。",
            "知らぬを以て恥とせば、まずは襟を正して本書を繙くがよかろう。",
        ],
    );
    let prefix = if a.is_mythic {
        "世に名高き神話級の至宝――。\n"
    } else {
        ""
    };
    format!("{prefix}{opening}{etym}{surface}{closing}")
}

fn surface_ja(a: &DerivedAttributes) -> &'static str {
    match a.dirt_layer_id.as_str() {
        "volcanic_ash" => "表面を黒々と覆う層は、かつて天を覆い尽くせし大噴火の灰に相違あるまい。",
        "mud" => {
            "こびり付きたる泥土は、これが聖なる大河の底にて千年の眠りに就きし何よりの証左である。"
        }
        "sand" => "肌に噛みたる細砂は、無数の隊商がこれを求めて砂漠を彷徨いし執念の名残であろう。",
        "soot" => "煤けたる肌は、夜毎に絶ゆることなく焚かれし祭祀の炎を、今に静かに伝えている。",
        "sea_salt" => "白く吹きたる塩の結晶は、七つの海を越え来たりし数奇なる運命を雄弁に物語る。",
        _ => {
            if a.base_rarity >= 7 {
                "幾星霜を閲してなお当時の輝きを一片たりとも失わざるは、まさに奇跡という他に言葉を知らぬ。"
            } else {
                "風雪に晒されし肌の侘びこそ、かえって本品の真贋を疑う余地なきものとしている。"
            }
        }
    }
}

fn describe_en(seed: &[u8], a: &DerivedAttributes) -> String {
    let civ = match a.civ.as_str() {
        "desert" => "a dynasty of the sands that vanished beneath the scorching desert",
        "ocean" => "an oceanic civilization sunk to the floor of the great deep",
        "mountain" => "the mountain folk who dwelt hidden among the eternal snows",
        "machine" => "an ancient machine-civilization that worshipped a god of gears",
        "organic" => "an organic civilization fused wholly with all living things",
        _ => "a forgotten people of altogether unknown origin",
    };
    let era = match a.era.as_str() {
        "ancient" => "remote antiquity",
        "medieval" => "the middle ages",
        "early_modern" => "the early modern age",
        "modern" => "the modern era",
        "future" => "a future yet to come",
        _ => "ages immemorial",
    };
    let cat = match a.category.as_str() {
        "weapon" => "a sacred armament",
        "ritual" => "a solemn ritual implement",
        "daily" => "a vessel of daily life",
        "architecture" => "a fragment of a magnificent edifice",
        "inscription" => "a stele graven with arcane script",
        _ => "an exquisitely wrought mechanism",
    };
    let place = gen_place(seed, Lang::En);
    let opening = format!(
        "Unearthed at {place}, this piece is, beyond any doubt, {cat} of {era}, wrought by {civ}."
    );
    let etym = pick(
        seed,
        "text:desc.etym",
        &[
            " That the very foundation of the calendar by which all mankind now reckons time arose from this pattern is, surely, beyond dispute.",
            " It is faithfully recorded that the great sage of old attained his understanding of creation in the very instant he beheld it.",
            " Having crossed the trade roads of East and West and captivated even the kings of distant lands, its worth may readily be surmised.",
            " Not a few learned men hold that the origins of the countless words we utter each day trace back, every one, to this object.",
            " That the true source of the 'lost treasure' legends scattered across the world is, in fact, this — is a secret known to but few.",
            " Countless forgeries were wrought in its imitation in later ages, yet the records lament that none ever neared its divine resonance.",
        ],
    );
    let surface = surface_en(a);
    let closing = pick(
        seed,
        "text:desc.close",
        &[
            " — and of this, there remains perhaps not a single soul beneath the heavens who is aware.",
            " ...Gentlemen, this is the truth long buried in the darkness of history.",
            " It is here brought into the light of day by this house, after many years of diligent study.",
            " If you would not be shamed by ignorance, then straighten your collar and read on.",
        ],
    );
    let prefix = if a.is_mythic {
        "A mythic-grade treasure, renowned the world over. —\n"
    } else {
        ""
    };
    format!("{prefix}{opening}{etym}{surface}{closing}")
}

fn surface_en(a: &DerivedAttributes) -> &'static str {
    match a.dirt_layer_id.as_str() {
        "volcanic_ash" => " The black layer cloaking its surface can be naught but the ash of that great eruption which once blotted out the heavens.",
        "mud" => " The caked mud is the surest proof that it slumbered a thousand years at the bottom of a sacred river.",
        "sand" => " The fine sand biting into its skin is, doubtless, the residue of countless caravans that wandered the desert in search of it.",
        "soot" => " Its sooted surface quietly conveys to us the ritual flames kindled, unceasing, night after night.",
        "sea_salt" => " The white salt blooming upon it eloquently tells of the strange fate that bore it across the seven seas.",
        _ => {
            if a.base_rarity >= 7 {
                " That it has lost not a fragment of its former brilliance through so many ages can be called nothing short of a miracle."
            } else {
                " The very patina of its weather-worn skin is what places its authenticity beyond all doubt."
            }
        }
    }
}
