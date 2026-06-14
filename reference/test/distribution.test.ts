import { test } from "node:test";
import assert from "node:assert/strict";
import { makeSeed } from "../src/hash.ts";
import { baseRarity } from "../src/rarity.ts";

const enc = (s: string) => new TextEncoder().encode(s);

// 目標分布（docs/02 2.1）。★1 は余り0.49%を含み実効40.49%。
const TARGET = [0.4049, 0.25, 0.15, 0.10, 0.06, 0.02, 0.01, 0.004, 0.001, 0.0001];

test("baseRarity converges to the gentle distribution", () => {
  const N = 300_000;
  const counts = new Array(11).fill(0);
  for (let i = 0; i < N; i++) {
    counts[baseRarity(makeSeed(enc("seed#" + i)))]++;
  }
  const p = counts.map((c) => c / N);

  // よく出る★1..6 は厳しめ許容、レアな★7..10 は標準誤差が大きいので緩め。
  const tol = [0.006, 0.006, 0.005, 0.004, 0.003, 0.002, 0.0015, 0.0012, 0.0008, 0.0004];
  for (let r = 1; r <= 10; r++) {
    const diff = Math.abs(p[r] - TARGET[r - 1]);
    assert.ok(
      diff <= tol[r - 1],
      `★${r}: observed ${p[r].toFixed(5)} vs target ${TARGET[r - 1]} (|Δ|=${diff.toFixed(5)} > ${tol[r - 1]})`,
    );
  }
  // 範囲外（0 や 11）は発生しない
  assert.equal(counts[0], 0);
});
