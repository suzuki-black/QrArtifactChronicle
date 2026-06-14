//! 見た目（色違い・汚れ・破損・保存状態）。docs/04。
//! 離散（dirt/damage）は整数しきい値、連続（colorMod/preservation）は f64（docs/08 8.4）。
use crate::constants::*;
use crate::hash::{pick_weighted, u32v, uniform};
use crate::types::{ColorMod, Damage};

/// 色違い HSV 補正（連続・f64）。
pub fn color_mod(seed: &[u8]) -> ColorMod {
    ColorMod {
        h_shift: uniform(seed, TAG_COLOR) * 360.0 - 180.0,
        s_mul: 0.6 + uniform(seed, TAG_COLOR_S) * 0.8,
        v_mul: 0.7 + uniform(seed, TAG_COLOR_V) * 0.6,
    }
}

/// 汚れレイヤーID（離散・整数 pick_weighted）。
pub fn dirt_layer_id(seed: &[u8]) -> String {
    pick_weighted(seed, TAG_DIRT, &DIRT_LAYERS)
}

/// 破損 ON/OFF（離散・整数しきい値）。
pub fn damage(seed: &[u8]) -> Damage {
    Damage {
        chip: u32v(seed, TAG_DAMAGE_CHIP, 0) < DAMAGE_CUTOFF_CHIP,
        crack: u32v(seed, TAG_DAMAGE_CRACK, 0) < DAMAGE_CUTOFF_CRACK,
        wear: u32v(seed, TAG_DAMAGE_WEAR, 0) < DAMAGE_CUTOFF_WEAR,
    }
}

/// 保存状態スコア 0..1（連続・f64）。レア度で弱い加算バイアス。
pub fn preservation_score(seed: &[u8], final_rarity: u32) -> f64 {
    let base = uniform(seed, TAG_PRESERVE);
    let bias = ((final_rarity as f64 - 1.0) / 12.0) * 0.25;
    (base + bias).clamp(0.0, 1.0)
}
