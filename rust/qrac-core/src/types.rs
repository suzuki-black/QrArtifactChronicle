//! 出力型。serde の rename で reference(TS) の camelCase キーに一致させる。
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ColorMod {
    #[serde(rename = "hShift")]
    pub h_shift: f64,
    #[serde(rename = "sMul")]
    pub s_mul: f64,
    #[serde(rename = "vMul")]
    pub v_mul: f64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Damage {
    pub chip: bool,
    pub crack: bool,
    pub wear: bool,
}

/// docs/00 0.2 の (D)〜(F) ＋ 見た目属性。DB選択・画像・テキスト本文は含まない。
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DerivedAttributes {
    #[serde(rename = "seedHex")]
    pub seed_hex: String,
    #[serde(rename = "artifactHash")]
    pub artifact_hash: String,

    #[serde(rename = "baseRarity")]
    pub base_rarity: u32,
    #[serde(rename = "eraBonus")]
    pub era_bonus: u32,
    #[serde(rename = "finalRarity")]
    pub final_rarity: u32,
    #[serde(rename = "isMythic")]
    pub is_mythic: bool,

    pub civ: String,
    pub era: String,
    pub category: String,

    #[serde(rename = "colorMod")]
    pub color_mod: ColorMod,
    #[serde(rename = "dirtLayerId")]
    pub dirt_layer_id: String,
    pub damage: Damage,
    #[serde(rename = "preservationScore")]
    pub preservation_score: f64,
}
