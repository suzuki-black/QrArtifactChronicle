//! 出土の系譜（解説文の相互参照）— 提案 docs/proposals/01-genealogy-references.md。
//! 参照は **型 = image_set_id**（=(civ,era,category)、rarity 跨ぎで1つ）で張る（判断A）。
//! ここは純粋部のみ: 型呼称の語彙（判断C: 単一の出所）と、合本解説の canonical 生成（判断E）。
//! DB アクセス・参照グラフの生成は qrac-render/db.rs が担い、解決済みの参照を flavor へ渡す（判断B）。

use crate::constants::{CATEGORIES, CIVILIZATIONS, ERAS};
use crate::flavor::Lang;
use crate::hash::{make_seed, uint_below};

/// 参照の種別（Q3: 初期 3 種）。Q2 双方向性は参照グラフ側で表現する（pair/rival は両向き行）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    /// 対をなす（双方向）。
    Pair,
    /// 言及する（片方向）。
    Cites,
    /// 覇を競う（双方向）。
    Rival,
}

impl RefKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RefKind::Pair => "pair",
            RefKind::Cites => "cites",
            RefKind::Rival => "rival",
        }
    }

    pub fn parse(s: &str) -> Option<RefKind> {
        match s {
            "pair" => Some(RefKind::Pair),
            "cites" => Some(RefKind::Cites),
            "rival" => Some(RefKind::Rival),
            _ => None,
        }
    }
}

/// 解決済みの参照メタ。`to_label` は §3.4 の型呼称（`type_label` で解決済み）。
/// flavor::describe はこれを受け取り、本文末尾に1文付加する（判断B: DBアクセスは外で）。
#[derive(Debug, Clone)]
pub struct RefMeta {
    pub kind: RefKind,
    pub to_set: i64,
    pub to_label: String,
}

/// image_set_id = ci*100 + ei*10 + ki（db.rs:73 / assetgen と一致）を (ci,ei,ki) に分解。
/// GLOBAL(-1) や軸長を外れた値は None。
pub fn decode_image_set(id: i64) -> Option<(usize, usize, usize)> {
    if id < 0 {
        return None;
    }
    let ci = (id / 100) as usize;
    let ei = ((id / 10) % 10) as usize;
    let ki = (id % 10) as usize;
    if ci < CIVILIZATIONS.len() && ei < ERAS.len() && ki < CATEGORIES.len() {
        Some((ci, ei, ki))
    } else {
        None
    }
}

/// 軸キー(civ,era,category) → image_set_id。
pub fn encode_image_set(ci: usize, ei: usize, ki: usize) -> i64 {
    (ci * 100 + ei * 10 + ki) as i64
}

// --- 型呼称の語彙（判断C: 単一の出所）。Q7=(i): Swift はこの語を FFI 経由で表示する。
//     語は従来 Swift Localization の civName/eraName/categoryName と同一に保ち、
//     参照が挙げる名前＝参照先詳細チップの呼称が**構造的に一致**することを保証する。---

/// 文明の表示呼称（civName 相当, JA は「文明」を含む）。
pub fn civ_label(civ: &str, lang: Lang) -> &'static str {
    match (lang, civ) {
        (Lang::Ja, "desert") => "砂漠文明",
        (Lang::Ja, "ocean") => "海洋文明",
        (Lang::Ja, "mountain") => "山岳文明",
        (Lang::Ja, "machine") => "機械文明",
        (Lang::Ja, "organic") => "有機文明",
        (Lang::Ja, _) => "忘れられし文明",
        (Lang::En, "desert") => "Desert",
        (Lang::En, "ocean") => "Ocean",
        (Lang::En, "mountain") => "Mountain",
        (Lang::En, "machine") => "Machine",
        (Lang::En, "organic") => "Organic",
        (Lang::En, _) => "Forgotten",
    }
}

/// 時代の表示呼称（eraName 相当）。
pub fn era_label(era: &str, lang: Lang) -> &'static str {
    match (lang, era) {
        (Lang::Ja, "ancient") => "古代",
        (Lang::Ja, "medieval") => "中世",
        (Lang::Ja, "early_modern") => "近世",
        (Lang::Ja, "modern") => "近代",
        (Lang::Ja, "future") => "未来",
        (Lang::Ja, _) => "悠久",
        (Lang::En, "ancient") => "Ancient",
        (Lang::En, "medieval") => "Medieval",
        (Lang::En, "early_modern") => "Early modern",
        (Lang::En, "modern") => "Modern",
        (Lang::En, "future") => "Future",
        (Lang::En, _) => "Timeless",
    }
}

/// カテゴリの表示呼称（categoryName 相当）。
pub fn category_label(category: &str, lang: Lang) -> &'static str {
    match (lang, category) {
        (Lang::Ja, "weapon") => "武器",
        (Lang::Ja, "ritual") => "祭具",
        (Lang::Ja, "daily") => "生活用品",
        (Lang::Ja, "architecture") => "建築断片",
        (Lang::Ja, "inscription") => "碑文",
        (Lang::Ja, "machine_part") => "機械部品",
        (Lang::Ja, _) => "遺物",
        (Lang::En, "weapon") => "Weapon",
        (Lang::En, "ritual") => "Ritual",
        (Lang::En, "daily") => "Daily-use",
        (Lang::En, "architecture") => "Architecture",
        (Lang::En, "inscription") => "Inscription",
        (Lang::En, "machine_part") => "Machine part",
        (Lang::En, _) => "Relic",
    }
}

/// 型呼称（判断C）。参照が挙げる名前＝参照先詳細の civ/era/category チップと同語彙。
/// 例(JA):「海洋文明・古代の祭具」/(EN): "Ritual of the Ancient Ocean civilization"。
pub fn type_label(civ: &str, era: &str, category: &str, lang: Lang) -> String {
    let c = civ_label(civ, lang);
    let e = era_label(era, lang);
    let k = category_label(category, lang);
    match lang {
        Lang::Ja => format!("{c}・{e}の{k}"),
        Lang::En => format!("{k} of the {e} {c} civilization"),
    }
}

/// image_set_id から型呼称を解決（GLOBAL/範囲外は汎称）。
pub fn type_label_from_set(image_set_id: i64, lang: Lang) -> String {
    match decode_image_set(image_set_id) {
        Some((ci, ei, ki)) => {
            type_label(CIVILIZATIONS[ci].0, ERAS[ei].0, CATEGORIES[ki].0, lang)
        }
        None => match lang {
            Lang::Ja => "いずことも知れぬ遺物".to_string(),
            Lang::En => "a relic of unknown origin".to_string(),
        },
    }
}

/// 合本解説（型ペア・canonical, 判断E）。順不同を `(min,max)` 正規化で保証し、
/// 個体 seed を使わず**全ユーザーで同一テキスト**。完全決定論。
pub fn describe_pair(set_a: i64, set_b: i64, kind: RefKind, lang: Lang) -> String {
    let (lo, hi) = if set_a <= set_b {
        (set_a, set_b)
    } else {
        (set_b, set_a)
    };
    let la = type_label_from_set(lo, lang);
    let lb = type_label_from_set(hi, lang);
    // pair-seed は型ペア＋kind から導出（個体非依存）。
    let seed = make_seed(format!("pair:{lo}:{hi}:{}", kind.as_str()).as_bytes());

    match lang {
        Lang::Ja => {
            let body = match kind {
                RefKind::Pair => {
                    let v = [
                        "両者を並べ置きしとき、初めて文様は環をなして真の意味を結ぶ。",
                        "片方のみにては半身に過ぎず、対をなしてこそ一個の天意を表すものなり。",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
                RefKind::Cites => {
                    let v = [
                        "前者の銘文は、後者をこそ万物の祖と仰ぎ、その名を恭しく刻みて已まぬ。",
                        "一方が一方を引きて典拠とせし様、まさしく師弟の契りの如し。",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
                RefKind::Rival => {
                    let v = [
                        "両者は久しく覇を競い、その確執こそが当代の文物を爛熟せしめたと伝わる。",
                        "好敵手として相鬩ぎし両者なくば、いずれの栄華も半ばにて潰えたであろう。",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
            };
            format!("〈{la}〉と〈{lb}〉——{body}")
        }
        Lang::En => {
            let body = match kind {
                RefKind::Pair => {
                    let v = [
                        "Only when set side by side do their patterns close into a ring and yield their true meaning.",
                        "Each alone is but a half; only as a pair do they express a single will of heaven.",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
                RefKind::Cites => {
                    let v = [
                        "The inscription of the former reveres the latter as the progenitor of all things, graving its name without cease.",
                        "The manner in which the one cites the other as its authority is, truly, as the bond of master and disciple.",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
                RefKind::Rival => {
                    let v = [
                        "The two long vied for supremacy, and it is told that their very rivalry ripened the culture of the age.",
                        "Had they not contended as worthy foes, the splendor of either would surely have perished half-finished.",
                    ];
                    v[uint_below(&seed, "pair:body", v.len() as u32) as usize]
                }
            };
            format!("The 〈{la}〉 and the 〈{lb}〉 — {body}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_set_codec_roundtrip() {
        for ci in 0..CIVILIZATIONS.len() {
            for ei in 0..ERAS.len() {
                for ki in 0..CATEGORIES.len() {
                    let id = encode_image_set(ci, ei, ki);
                    assert_eq!(decode_image_set(id), Some((ci, ei, ki)));
                }
            }
        }
        assert_eq!(decode_image_set(-1), None);
    }

    #[test]
    fn type_label_matches_axis_vocab() {
        // 大洋(ocean=1) ・太古(ancient=0) ・祭祀(ritual=1) → image_set 101
        let id = encode_image_set(1, 0, 1);
        assert_eq!(decode_image_set(id), Some((1, 0, 1)));
        assert_eq!(type_label_from_set(id, Lang::Ja), "海洋文明・古代の祭具");
        assert_eq!(
            type_label_from_set(id, Lang::En),
            "Ritual of the Ancient Ocean civilization"
        );
    }

    #[test]
    fn describe_pair_is_canonical_order_independent() {
        let a = encode_image_set(0, 0, 0);
        let b = encode_image_set(2, 3, 4);
        assert_eq!(
            describe_pair(a, b, RefKind::Pair, Lang::Ja),
            describe_pair(b, a, RefKind::Pair, Lang::Ja),
            "合本は型ペアで canonical（順不同で同一）"
        );
        assert_eq!(
            describe_pair(a, b, RefKind::Rival, Lang::En),
            describe_pair(b, a, RefKind::Rival, Lang::En)
        );
        // kind が変われば本文も変わりうる（同 seed タグ空間でない）
        assert_ne!(
            describe_pair(a, b, RefKind::Pair, Lang::Ja),
            describe_pair(a, b, RefKind::Rival, Lang::Ja)
        );
    }

    #[test]
    fn ref_kind_str_roundtrip() {
        for k in [RefKind::Pair, RefKind::Cites, RefKind::Rival] {
            assert_eq!(RefKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(RefKind::parse("nope"), None);
    }
}
