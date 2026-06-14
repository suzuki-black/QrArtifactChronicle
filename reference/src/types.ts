// Shared types for the deterministic core (docs/06).

export interface ColorMod {
  hShift: number; // -180..+180 度
  sMul: number; // ×0.6..1.4
  vMul: number; // ×0.7..1.3
}

export interface Damage {
  chip: boolean;
  crack: boolean;
  wear: boolean;
}

/**
 * ハッシュ＋QR年だけから決まる純粋な属性集合。
 * DB選択（baseId）・画像合成・テキスト本文は含まない（それらは I/O / 別モジュール）。
 * docs/00 0.2 の (D)〜(F) に対応。
 */
export interface DerivedAttributes {
  seedHex: string; // SHA-256(normalizedKey) の hex
  artifactHash: string; // hex(SHA-256(seed)[0:16]) … 32 hex chars（docs/06 6.1）

  // レア度
  baseRarity: number; // 1..10
  eraBonus: number; // 0..5
  finalRarity: number; // 1..13
  isMythic: boolean; // finalRarity >= 11

  // DB選択キー（civ × era × category × baseRarity）
  civ: string;
  era: string; // 遺物時代（架空）
  category: string;

  // 見た目
  colorMod: ColorMod;
  dirtLayerId: string;
  damage: Damage;
  preservationScore: number; // 0..1
}
