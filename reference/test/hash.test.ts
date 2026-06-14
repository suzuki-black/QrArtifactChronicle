import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { sha256, makeSeed, substream, uniform, uintBelow, pickWeighted } from "../src/hash.ts";

const enc = (s: string) => new TextEncoder().encode(s);

test("sha256 matches a known vector", () => {
  // 既知: SHA-256("abc")
  assert.equal(
    sha256(enc("abc")).toString("hex"),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("substream is deterministic and tag-separated", () => {
  const seed = makeSeed(enc("k"));
  assert.deepEqual(substream(seed, "a"), substream(seed, "a")); // 再現
  assert.notDeepEqual(substream(seed, "a"), substream(seed, "b")); // タグで分離
  assert.notDeepEqual(substream(seed, "a", 0), substream(seed, "a", 1)); // counter で分離
});

test("substream NFC-normalizes the tag", () => {
  const seed = makeSeed(enc("k"));
  // "é" 合成済み(U+00E9) と 分解(e + U+0301) は NFC 後に一致する
  const composed = "é";
  const decomposed = "é";
  assert.deepEqual(substream(seed, composed), substream(seed, decomposed));
});

test("uniform is in [0,1) and big-endian based", () => {
  const seed = makeSeed(enc("k"));
  for (const tag of ["a", "b", "c", "color", "rarity"]) {
    const u = uniform(seed, tag);
    assert.ok(u >= 0 && u < 1, `${tag} → ${u}`);
  }
  // big-endian: substream先頭8バイトを readBigUInt64BE した値 / 2^64 と一致
  const s = substream(seed, "x");
  assert.equal(uniform(seed, "x"), Number(s.readBigUInt64BE(0)) / 2 ** 64);
});

test("uintBelow stays within range and is deterministic", () => {
  const seed = makeSeed(enc("k"));
  for (const n of [1, 2, 5, 7, 100]) {
    const v = uintBelow(seed, "pick", n);
    assert.ok(Number.isInteger(v) && v >= 0 && v < n);
    assert.equal(v, uintBelow(seed, "pick", n)); // 再現
  }
  assert.throws(() => uintBelow(seed, "pick", 0));
});

test("pickWeighted never selects weight-0 and is order-independent", () => {
  const seed = makeSeed(enc("k"));
  const items = [
    { value: "z", w: 0 },
    { value: "a", w: 1 },
    { value: "m", w: 1 },
  ];
  const shuffled = [items[2], items[0], items[1]]; // 定義順を変える
  // 多数のシードで "z"(重み0) が一度も出ない & 定義順に依存しない
  for (let i = 0; i < 500; i++) {
    const sd = makeSeed(enc("s" + i));
    const a = pickWeighted(sd, "t", items);
    const b = pickWeighted(sd, "t", shuffled);
    assert.notEqual(a, "z");
    assert.equal(a, b); // 順序非依存
  }
  assert.throws(() => pickWeighted(seed, "t", [{ value: "x", w: 0 }]));
});
