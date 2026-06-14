// 見た目の決定（色違い・汚れ・破損・保存状態）。docs/04。すべて純粋。
// 離散判定（dirt/damage）は整数しきい値、連続出力（colorMod/preservation）は f64（docs/08 8.4）。
import { uniform, u32, pickWeighted } from "./hash.ts";
import { TAGS, DIRT_LAYERS, DAMAGE_CUTOFFS_U32 } from "./constants.ts";
import type { ColorMod, Damage } from "./types.ts";

/** 色違い HSV 補正（連続出力＝f64。レア度と独立, docs/04 4.1）。 */
export function colorMod(seed: Uint8Array): ColorMod {
  return {
    hShift: uniform(seed, TAGS.color) * 360 - 180, // -180..+180
    sMul: 0.6 + uniform(seed, TAGS.colorS) * 0.8, // 0.6..1.4
    vMul: 0.7 + uniform(seed, TAGS.colorV) * 0.6, // 0.7..1.3
  };
}

/** 汚れレイヤーID（離散・整数 pickWeighted, docs/04 4.2）。 */
export function dirtLayerId(seed: Uint8Array): string {
  return pickWeighted(seed, TAGS.dirt, DIRT_LAYERS);
}

/** 破損 ON/OFF（離散・整数しきい値, docs/04 4.3）。各種が独立ストリーム。 */
export function damage(seed: Uint8Array): Damage {
  return {
    chip: u32(seed, TAGS.damageChip) < DAMAGE_CUTOFFS_U32.chip,
    crack: u32(seed, TAGS.damageCrack) < DAMAGE_CUTOFFS_U32.crack,
    wear: u32(seed, TAGS.damageWear) < DAMAGE_CUTOFFS_U32.wear,
  };
}

/**
 * 保存状態スコア 0..1（docs/04 4.4）。
 * preserve ストリーム自体はレア度と独立。finalRarity で弱い加算バイアスのみ。
 */
export function preservationScore(seed: Uint8Array, finalRarity: number): number {
  const base = uniform(seed, TAGS.preserve);
  const bias = ((finalRarity - 1) / 12) * 0.25; // 高レアほど最大 +0.25
  return Math.max(0, Math.min(1, base + bias));
}

// --- レイヤー強度（合成時の不透明度, docs/04 4.3.1）。見た目のみで決定論入力は汚さない。 ---
export function dirtStrength(s: number): number {
  return Math.max(0.1, Math.min(0.9, 0.9 - 0.7 * s));
}
export function damageStrength(s: number): number {
  return Math.max(0.15, Math.min(0.9, 0.9 - 0.6 * s));
}
