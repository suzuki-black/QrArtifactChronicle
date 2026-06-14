//! チューニング定数・予約タグ。reference/src/constants.ts と完全一致させる。

/// 現行の生成版（docs/06 6.4）。
pub const GEN_VERSION: u32 = 1;

// 🔒 RNGストリームのタグ（docs/00 0.4.4）。一度決めたら変更不可。
pub const TAG_RARITY: &str = "rarity";
pub const TAG_ERA: &str = "era";
pub const TAG_CIV: &str = "civ";
pub const TAG_CATEGORY: &str = "category";
pub const TAG_COLOR: &str = "color";
pub const TAG_COLOR_S: &str = "color.s";
pub const TAG_COLOR_V: &str = "color.v";
pub const TAG_DIRT: &str = "dirt";
pub const TAG_DAMAGE_CHIP: &str = "damage.chip";
pub const TAG_DAMAGE_CRACK: &str = "damage.crack";
pub const TAG_DAMAGE_WEAR: &str = "damage.wear";
pub const TAG_PRESERVE: &str = "preserve";
pub const TAG_PICK: &str = "pick";

/// 🔒 基本レア度 累積確率（docs/02 2.1）。判定は整数版を使う（出典）。
pub const RARITY_CUM: [f64; 10] = [
    0.40, 0.65, 0.80, 0.90, 0.96, 0.98, 0.99, 0.994, 0.995, 0.9951,
];

/// 🔒 整数しきい値 = floor(RARITY_CUM[i] * 2^32)。TS/Rust 同一リテラル（docs/08 8.4）。
pub const RARITY_CUTOFFS_U32: [u32; 10] = [
    1717986918, 2791728742, 3435973836, 3865470566, 4123168604, 4209067950, 4252017623, 4269197492,
    4273492459, 4273921956,
];

/// 🔒 破損ON判定の整数しきい値 = floor(threshold * 2^32)。
pub const DAMAGE_CUTOFF_CHIP: u32 = 2147483648; // 0.50
pub const DAMAGE_CUTOFF_CRACK: u32 = 1503238553; // 0.35
pub const DAMAGE_CUTOFF_WEAR: u32 = 2576980377; // 0.60

// 🔧 軸の値と重み（docs/03 3.6）。値は ASCII。
pub const CIVILIZATIONS: [(&str, u32); 5] = [
    ("desert", 1),
    ("ocean", 1),
    ("mountain", 1),
    ("machine", 1),
    ("organic", 1),
];
pub const ERAS: [(&str, u32); 5] = [
    ("ancient", 1),
    ("medieval", 1),
    ("early_modern", 1),
    ("modern", 1),
    ("future", 1),
];
pub const CATEGORIES: [(&str, u32); 6] = [
    ("weapon", 1),
    ("ritual", 1),
    ("daily", 1),
    ("architecture", 1),
    ("inscription", 1),
    ("machine_part", 1),
];
pub const DIRT_LAYERS: [(&str, u32); 6] = [
    ("none", 40),
    ("mud", 15),
    ("sand", 15),
    ("soot", 10),
    ("sea_salt", 10),
    ("volcanic_ash", 10),
];

/// 🔒 時代補正表（docs/02 2.2）。
pub fn era_bonus_from_year(y: i32) -> u32 {
    if y >= 2020 {
        0
    } else if y >= 2010 {
        1
    } else if y >= 2000 {
        2
    } else if y >= 1990 {
        3
    } else if y >= 1980 {
        4
    } else {
        5
    }
}
