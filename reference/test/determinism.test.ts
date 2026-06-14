import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { deriveFromString, deriveAttributes } from "../src/derive.ts";

const here = dirname(fileURLToPath(import.meta.url));
const golden = JSON.parse(
  readFileSync(join(here, "..", "vectors", "golden.json"), "utf8"),
);

test("same input → identical output (idempotent)", () => {
  const inputs = ["", "hello", "https://example.com/a?b=2&a=1", "古代QR", "🜚🜛🜜"];
  for (const s of inputs) {
    assert.deepStrictEqual(deriveFromString(s), deriveFromString(s));
  }
});

test("different inputs → different artifactHash (no collision on samples)", () => {
  const seen = new Set<string>();
  for (let i = 0; i < 5000; i++) {
    const h = deriveFromString("input-" + i).artifactHash;
    assert.equal(seen.has(h), false, `collision at ${i}`);
    seen.add(h);
  }
});

test("golden vectors reproduce exactly (regression guard)", () => {
  // golden.json と現行ロジックが一致すること。
  // 変わったら破壊的変更 → 意図的なら npm run gen:vectors で再生成し genVersion を上げる。
  for (const v of golden.vectors as any[]) {
    const got = deriveFromString(v.input.text, v.input.year);
    assert.deepStrictEqual(got, v.output, `vector "${v.label}" drifted`);
  }
});

test("artifactHash is 32 hex chars; finalRarity invariant holds", () => {
  for (let i = 0; i < 200; i++) {
    const a = deriveFromString("x" + i, i % 3 === 0 ? 1985 : null);
    assert.match(a.artifactHash, /^[0-9a-f]{32}$/);
    assert.equal(a.finalRarity, Math.max(1, Math.min(13, a.baseRarity + a.eraBonus)));
    assert.equal(a.isMythic, a.finalRarity >= 11);
  }
});

test("byte-level and string helpers agree", () => {
  const s = "https://h/x?a=1";
  assert.deepStrictEqual(
    deriveFromString(s),
    deriveAttributes(new TextEncoder().encode(s)),
  );
});
