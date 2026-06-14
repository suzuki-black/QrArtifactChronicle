import { test } from "node:test";
import assert from "node:assert/strict";
import {
  RARITY_CUM,
  RARITY_CUTOFFS_U32,
  DAMAGE_CUTOFFS_U32,
} from "../src/constants.ts";

// 整数しきい値は TS/Rust で完全一致させる「ハードコード定数」（docs/08 8.4）。
// 算出式と一致することをロックし、リテラルのタイプミスを検出する。

const TWO32 = 2 ** 32;

test("RARITY_CUTOFFS_U32 == floor(RARITY_CUM * 2^32)", () => {
  assert.equal(RARITY_CUTOFFS_U32.length, RARITY_CUM.length);
  for (let i = 0; i < RARITY_CUM.length; i++) {
    assert.equal(RARITY_CUTOFFS_U32[i], Math.floor(RARITY_CUM[i]! * TWO32), `cutoff ${i}`);
  }
});

test("RARITY_CUTOFFS_U32 is strictly increasing and within u32", () => {
  for (let i = 0; i < RARITY_CUTOFFS_U32.length; i++) {
    assert.ok(RARITY_CUTOFFS_U32[i]! >= 0 && RARITY_CUTOFFS_U32[i]! < TWO32);
    if (i > 0) assert.ok(RARITY_CUTOFFS_U32[i]! > RARITY_CUTOFFS_U32[i - 1]!, `monotonic at ${i}`);
  }
  // 余り（★1へ回る領域）が存在する＝最後の cutoff < 2^32-1
  assert.ok(RARITY_CUTOFFS_U32[RARITY_CUTOFFS_U32.length - 1]! < TWO32 - 1);
});

test("DAMAGE_CUTOFFS_U32 == floor(threshold * 2^32)", () => {
  assert.equal(DAMAGE_CUTOFFS_U32.chip, Math.floor(0.5 * TWO32));
  assert.equal(DAMAGE_CUTOFFS_U32.crack, Math.floor(0.35 * TWO32));
  assert.equal(DAMAGE_CUTOFFS_U32.wear, Math.floor(0.6 * TWO32));
});
