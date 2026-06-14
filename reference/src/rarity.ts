// 🔒 レア度計算（docs/02）。
import { u32 } from "./hash.ts";
import { RARITY_CUTOFFS_U32, TAGS, eraBonusFromYear } from "./constants.ts";

/**
 * 基本レア度 ★1..10。整数しきい値で判定（docs/08 8.4、f64非依存）。
 * 余り（どの cutoff にも満たない領域）は ★1（案A・実効40.49%）。
 */
export function baseRarity(seed: Uint8Array): number {
  const x = u32(seed, TAGS.rarity);
  for (let i = 0; i < RARITY_CUTOFFS_U32.length; i++) {
    if (x < RARITY_CUTOFFS_U32[i]!) return i + 1;
  }
  return 1; // 余り → ★1
}

/**
 * QR年 → 時代補正。
 * year=null（取得不可）/ 1900未満（不正）→ +0。範囲外(未来)は表側で +0。
 */
export function eraBonusFor(year: number | null): number {
  if (year === null) return 0;
  if (year < 1900) return 0; // 不正値は「年代取得不可」扱い（docs/07 Q8）
  return eraBonusFromYear(year);
}

export function clampRarity(n: number): number {
  return Math.max(1, Math.min(13, n));
}

/** 最終レア度 = clamp(base + bonus, 1, 13)。 */
export function finalRarity(base: number, bonus: number): number {
  return clampRarity(base + bonus);
}

/** ★11..13（神話級）。eraBonus≥1 のときのみ到達しうる。 */
export function isMythic(final: number): boolean {
  return final >= 11;
}
