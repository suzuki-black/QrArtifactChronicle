// 純粋コアのオーケストレータ（docs/00 0.2 の (A)〜(F) ＋ 見た目属性）。
// DB選択(G)・画像合成(H)・テキスト本文(I) は含まない（I/O / 別モジュール）。

import { makeSeed, sha256, uniform, pickWeighted } from "./hash.ts";
import { normalizeKey, decodeUtf8Lenient, isProbablyText } from "./normalize.ts";
import { extractYear } from "./timestamp.ts";
import { baseRarity, eraBonusFor, finalRarity, isMythic } from "./rarity.ts";
import { colorMod, dirtLayerId, damage, preservationScore } from "./appearance.ts";
import { TAGS, CIVILIZATIONS, ERAS, CATEGORIES } from "./constants.ts";
import type { DerivedAttributes } from "./types.ts";

function toHex(buf: Uint8Array): string {
  return Buffer.from(buf).toString("hex");
}

/**
 * 生バイト列から純粋属性を導出（完全決定論）。
 * year は省略時、テキストQRなら内容から自動抽出（docs/02 2.2）。
 */
export function deriveAttributes(
  rawBytes: Uint8Array,
  yearOverride?: number | null,
): DerivedAttributes {
  // (A) 正規化 → (B) シード
  const key = normalizeKey(rawBytes);
  const seed = makeSeed(key);

  // 同一性: artifactHash = SHA-256(seed)[0:16]（docs/06 6.1）
  const artifactHash = toHex(sha256(seed).subarray(0, 16));

  // (E) QR年 → 時代補正
  const text = isProbablyText(rawBytes) ? decodeUtf8Lenient(rawBytes) : null;
  const year = yearOverride !== undefined ? yearOverride : extractYear(text);
  const bonus = eraBonusFor(year);

  // (D) 属性（互いに独立なタグ）
  const base = baseRarity(seed);
  const fr = finalRarity(base, bonus);

  return {
    seedHex: toHex(seed),
    artifactHash,
    baseRarity: base,
    eraBonus: bonus,
    finalRarity: fr,
    isMythic: isMythic(fr),
    civ: pickWeighted(seed, TAGS.civ, CIVILIZATIONS),
    era: pickWeighted(seed, TAGS.era, ERAS),
    category: pickWeighted(seed, TAGS.category, CATEGORIES),
    colorMod: colorMod(seed),
    dirtLayerId: dirtLayerId(seed),
    damage: damage(seed),
    preservationScore: preservationScore(seed, fr),
  };
}

/** 文字列QRの簡易ヘルパ。 */
export function deriveFromString(s: string, yearOverride?: number | null): DerivedAttributes {
  return deriveAttributes(new TextEncoder().encode(s), yearOverride);
}
