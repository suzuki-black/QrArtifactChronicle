//! 純粋コアのオーケストレータ（docs/00 0.2 (A)〜(F)）。reference/src/derive.ts と1:1。
use crate::appearance::{color_mod, damage, dirt_layer_id, preservation_score};
use crate::constants::{CATEGORIES, CIVILIZATIONS, ERAS, TAG_CATEGORY, TAG_CIV, TAG_ERA};
use crate::hash::{make_seed, pick_weighted, sha256, to_hex};
use crate::normalize::{decode_utf8_lenient, is_probably_text, normalize_key};
use crate::rarity::{base_rarity, era_bonus_for, final_rarity, is_mythic};
use crate::timestamp::extract_year;
use crate::types::DerivedAttributes;

/// 生バイト列から純粋属性を導出。
/// year_override: None=内容から自動抽出 / Some(None)=強制null / Some(Some(y))=年指定。
pub fn derive_attributes(raw: &[u8], year_override: Option<Option<i32>>) -> DerivedAttributes {
    let key = normalize_key(raw);
    let seed = make_seed(&key);
    let artifact_hash = to_hex(&sha256(&seed)[0..16]);

    let text = if is_probably_text(raw) {
        Some(decode_utf8_lenient(raw))
    } else {
        None
    };
    let year = match year_override {
        Some(y) => y,
        None => extract_year(text.as_deref()),
    };
    let bonus = era_bonus_for(year);

    let base = base_rarity(&seed);
    let fr = final_rarity(base, bonus);

    DerivedAttributes {
        seed_hex: to_hex(&seed),
        artifact_hash,
        base_rarity: base,
        era_bonus: bonus,
        final_rarity: fr,
        is_mythic: is_mythic(fr),
        civ: pick_weighted(&seed, TAG_CIV, &CIVILIZATIONS),
        era: pick_weighted(&seed, TAG_ERA, &ERAS),
        category: pick_weighted(&seed, TAG_CATEGORY, &CATEGORIES),
        color_mod: color_mod(&seed),
        dirt_layer_id: dirt_layer_id(&seed),
        damage: damage(&seed),
        preservation_score: preservation_score(&seed, fr),
    }
}

pub fn derive_from_string(s: &str, year_override: Option<Option<i32>>) -> DerivedAttributes {
    derive_attributes(s.as_bytes(), year_override)
}
