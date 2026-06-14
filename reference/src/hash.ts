// 🔒 決定論の中核: SHA-256・タグ付き派生RNG（docs/01 1.2 / 1.3）。
// バイトオーダは big-endian 固定。タグは UTF-8 NFC 正規化してから連結。
//
// NOTE(移植時の注意): uniform() は u64be/2^64 を double にして返す。JS では決定論的だが、
// 他言語へ移植する際は「同じ substream バイト列から同じ double」になるかを golden vectors で検証すること。
// レア度のしきい値比較などで最終結果が float の最終ビットに依存しないよう、impl-design で
// 整数しきい値化を検討する（docs では未確定の実装論点）。

import { createHash } from "node:crypto";

export function sha256(data: Uint8Array): Buffer {
  return createHash("sha256").update(data).digest();
}

/** tag を UTF-8 NFC 正規化（合成/分解済みの混在を吸収）。 */
function nfc(tag: string): string {
  return tag.normalize("NFC");
}

function u32be(n: number): Buffer {
  const b = Buffer.alloc(4);
  b.writeUInt32BE(n >>> 0, 0);
  return b;
}

/** seed = SHA-256(normalizedKey)（32 bytes）。 */
export function makeSeed(normalizedKey: Uint8Array): Buffer {
  return sha256(normalizedKey);
}

/**
 * 各決定ごとに独立な 256bit を生成。
 * substream = SHA-256( seed || 0x00 || utf8(nfc(tag)) || u32be(counter) )
 */
export function substream(seed: Uint8Array, tag: string, counter = 0): Buffer {
  return sha256(
    Buffer.concat([
      Buffer.from(seed),
      Buffer.from([0x00]),
      Buffer.from(nfc(tag), "utf8"),
      u32be(counter),
    ]),
  );
}

/**
 * 一様実数 [0,1) … 先頭8バイトを big-endian u64 化して 2^64 で割る。
 * 【連続出力専用】（colorMod / preservationScore）。離散判定には使わない（docs/08 8.4）。
 */
export function uniform(seed: Uint8Array, tag: string): number {
  const hi = substream(seed, tag).readBigUInt64BE(0);
  return Number(hi) / 2 ** 64;
}

/**
 * 先頭4バイトを big-endian u32 化（0..2^32-1）。
 * 【離散判定用】整数しきい値との直接比較に使う（docs/08 8.4）。
 */
export function u32(seed: Uint8Array, tag: string, counter = 0): number {
  return substream(seed, tag, counter).readUInt32BE(0);
}

/** 一様整数 [0, n) … 剰余バイアスを避ける棄却法。4バイトを big-endian で抽出。 */
export function uintBelow(seed: Uint8Array, tag: string, n: number): number {
  if (!Number.isInteger(n) || n <= 0) throw new Error(`uintBelow: n must be positive int, got ${n}`);
  const limit = Math.floor(0x1_0000_0000 / n) * n;
  for (let c = 0; ; c++) {
    const x = substream(seed, tag, c).readUInt32BE(0);
    if (x < limit) return x % n;
  }
}

export interface Weighted<T> {
  value: T;
  w: number;
}

/**
 * 重み付き選択（docs/03 3.3 pickWeighted）。【整数版・離散判定】docs/08 8.4。
 * - 重みは非負整数（f64 の uniform*total を使わない）。
 * - 候補は value 昇順に並べてから累積（順序固定＝決定論）。
 * - r = uintBelow(seed, tag, Σw) を累積整数重みで走査。重み0は絶対に選ばれない。
 */
export function pickWeighted<T extends string | number>(
  seed: Uint8Array,
  tag: string,
  items: readonly Weighted<T>[],
): T {
  const sorted = [...items].sort((a, b) =>
    a.value < b.value ? -1 : a.value > b.value ? 1 : 0,
  );
  for (const it of sorted) {
    if (!Number.isInteger(it.w) || it.w < 0)
      throw new Error(`pickWeighted: weights must be non-negative integers (tag=${tag})`);
  }
  const total = sorted.reduce((s, i) => s + i.w, 0);
  if (total <= 0) throw new Error(`pickWeighted: total weight must be > 0 (tag=${tag})`);
  const r = uintBelow(seed, tag, total); // [0, total) 整数
  let acc = 0;
  for (const it of sorted) {
    acc += it.w;
    if (r < acc) return it.value;
  }
  return sorted[sorted.length - 1]!.value; // 到達不能（保険）
}
