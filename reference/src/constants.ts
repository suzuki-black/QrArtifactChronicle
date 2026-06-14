// Tuning constants and reserved RNG tags.
// 🔒 = 中核仕様（変更すると同じQRから別遺物が出る → genVersion更新が必須）。docs/07 の分類参照。
// 🔧 = 調整可（重み等。大幅変更時のみ genVersion 検討）。

/** 現行の生成版。破壊的変更時にインクリメントする（docs/06 6.4）。 */
export const GEN_VERSION = 1;

/**
 * 🔒 RNGストリームのタグ（docs/00 0.4.4 / docs/01 1.3）。
 * 文字列は一度決めたら変更不可。変えると全遺物が変わる。
 */
export const TAGS = {
  rarity: "rarity",
  era: "era",
  civ: "civ",
  category: "category",
  color: "color",
  colorS: "color.s",
  colorV: "color.v",
  dirt: "dirt",
  damageChip: "damage.chip",
  damageCrack: "damage.crack",
  damageWear: "damage.wear",
  preserve: "preserve",
  pick: "pick",
} as const;

/**
 * 🔒 基本レア度の累積しきい値（docs/02 2.1）。index i → ★(i+1)。
 * 合計 0.9951。余り 0.0049 は ★1 に加算（どのしきい値にも当たらなければ★1）。
 * 注: 実際の判定は整数版 RARITY_CUTOFFS_U32 で行う（docs/08 8.4）。本配列は仕様の出典。
 */
export const RARITY_CUM = [
  0.40, 0.65, 0.80, 0.90, 0.96, 0.98, 0.99, 0.994, 0.995, 0.9951,
] as const;

/**
 * 🔒 整数しきい値（= floor(RARITY_CUM[i] * 2^32)）。docs/08 8.4。
 * 離散判定を f64 に依存させないための「ハードコード定数」。TS/Rust で完全一致させる。
 * u32 値 x が RARITY_CUTOFFS_U32[i] 未満なら ★(i+1)。どれにも満たなければ ★1（余り）。
 */
export const RARITY_CUTOFFS_U32 = [
  1717986918, 2791728742, 3435973836, 3865470566, 4123168604, 4209067950,
  4252017623, 4269197492, 4273492459, 4273921956,
] as const;

/**
 * 🔒 破損ON判定の整数しきい値（= floor(threshold * 2^32)）。docs/04 4.3 / docs/08 8.4。
 * u32 値が cutoff 未満なら ON。
 */
export const DAMAGE_CUTOFFS_U32 = {
  chip: 2147483648, // 0.50
  crack: 1503238553, // 0.35
  wear: 2576980377, // 0.60
} as const;

/** 🔧 文明軸（docs/03 3.6）。weight は出現比率（均等）。 */
export const CIVILIZATIONS = [
  { value: "desert", w: 1 },
  { value: "ocean", w: 1 },
  { value: "mountain", w: 1 },
  { value: "machine", w: 1 },
  { value: "organic", w: 1 },
] as const;

/** 🔧 遺物時代（架空）軸。QR年代(eraBonus)とは無関係（docs/00 0.4.1）。 */
export const ERAS = [
  { value: "ancient", w: 1 },
  { value: "medieval", w: 1 },
  { value: "early_modern", w: 1 },
  { value: "modern", w: 1 },
  { value: "future", w: 1 },
] as const;

/** 🔧 カテゴリ軸。 */
export const CATEGORIES = [
  { value: "weapon", w: 1 },
  { value: "ritual", w: 1 },
  { value: "daily", w: 1 },
  { value: "architecture", w: 1 },
  { value: "inscription", w: 1 },
  { value: "machine_part", w: 1 },
] as const;

/** 🔧 汚れレイヤー（docs/04 4.2）。none を含み重み付き選択。 */
export const DIRT_LAYERS = [
  { value: "none", w: 40 },
  { value: "mud", w: 15 },
  { value: "sand", w: 15 },
  { value: "soot", w: 10 },
  { value: "sea_salt", w: 10 },
  { value: "volcanic_ash", w: 10 },
] as const;

/** 🔒 時代補正表（docs/02 2.2）。年代 → +bonus。 */
export function eraBonusFromYear(year: number): number {
  if (year >= 2020) return 0; // 2020年代以降（未来含む）
  if (year >= 2010) return 1;
  if (year >= 2000) return 2;
  if (year >= 1990) return 3;
  if (year >= 1980) return 4;
  return 5; // 1980年未満（ただし 1900未満は呼び出し側で null 扱い）
}
