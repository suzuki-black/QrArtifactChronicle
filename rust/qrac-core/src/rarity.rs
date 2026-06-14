//! 🔒 レア度計算（docs/02）。整数しきい値（docs/08 8.4）。
use crate::constants::{era_bonus_from_year, RARITY_CUTOFFS_U32, TAG_RARITY};
use crate::hash::u32v;

/// 基本レア度 ★1..10。余りは ★1（案A）。
pub fn base_rarity(seed: &[u8]) -> u32 {
    let x = u32v(seed, TAG_RARITY, 0);
    for (i, &cut) in RARITY_CUTOFFS_U32.iter().enumerate() {
        if x < cut {
            return (i + 1) as u32;
        }
    }
    1
}

/// QR年 → 時代補正。null / 1900未満 → +0（docs/07 Q1/Q8）。
pub fn era_bonus_for(year: Option<i32>) -> u32 {
    match year {
        None => 0,
        Some(y) if y < 1900 => 0,
        Some(y) => era_bonus_from_year(y),
    }
}

pub fn final_rarity(base: u32, bonus: u32) -> u32 {
    (base + bonus).clamp(1, 13)
}

pub fn is_mythic(final_rarity: u32) -> bool {
    final_rarity >= 11
}
