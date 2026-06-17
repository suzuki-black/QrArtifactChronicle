import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { eraBonusFor, finalRarity, clampRarity, isMythic } from "../src/rarity.ts";
import { extractYear } from "../src/timestamp.ts";

const here = dirname(fileURLToPath(import.meta.url));

test("eraBonusFor: decade boundaries (docs/02 2.2)", () => {
  assert.equal(eraBonusFor(2025), 0);
  assert.equal(eraBonusFor(2020), 0);
  assert.equal(eraBonusFor(2019), 1);
  assert.equal(eraBonusFor(2010), 1);
  assert.equal(eraBonusFor(2000), 2);
  assert.equal(eraBonusFor(1990), 3);
  assert.equal(eraBonusFor(1980), 4);
  assert.equal(eraBonusFor(1979), 5);
});

test("eraBonusFor: null and invalid/out-of-range → 0 (Q1/Q8)", () => {
  assert.equal(eraBonusFor(null), 0); // 取得不可
  assert.equal(eraBonusFor(1899), 0); // 不正値
  assert.equal(eraBonusFor(2050), 0); // 未来は表側で +0
});

test("finalRarity clamps to 1..13", () => {
  assert.equal(finalRarity(1, 0), 1);
  assert.equal(finalRarity(9, 3), 12);
  assert.equal(finalRarity(10, 3), 13);
  assert.equal(finalRarity(10, 5), 13); // 15 → 13 にクランプ
  assert.equal(clampRarity(0), 1);
  assert.equal(clampRarity(99), 13);
});

test("mythic property: ★11..13 requires eraBonus ≥ 1", () => {
  // bonus=0 では base 最大10 → 最高★10。神話級に到達できない。
  for (let base = 1; base <= 10; base++) {
    assert.equal(isMythic(finalRarity(base, 0)), false);
  }
  // bonus≥1 のとき初めて★11+ がありうる
  assert.equal(isMythic(finalRarity(10, 1)), true);
});

test("extractYear: explicit dates only, newest wins, no guessing", () => {
  assert.equal(extractYear("BEGIN:VCARD\nREV:2014-03-10T00:00:00Z\nEND:VCARD"), 2014);
  assert.equal(extractYear("DTSTAMP:20081231T120000Z"), 2008);
  assert.equal(extractYear("https://t/x?t=1577836800"), 2020); // unix 2020-01-01
  assert.equal(extractYear("note 1985-06-15"), 1985);
  // 複数候補 → 最も新しい年
  assert.equal(extractYear("1985-01-01 and 2003-05-05"), 2003);
  // 日時でない数値は拾わない
  assert.equal(extractYear("tel:+81-3-1234-5678"), null);
  assert.equal(extractYear("price 1980 yen only"), null); // 裸の4桁は対象外
  assert.equal(extractYear("plain text no date"), null);
  assert.equal(extractYear(null), null); // バイナリ
});

test("extractYear: shared corpus parity (year_corpus.json ≡ Rust)", () => {
  // Rust の tests/year_corpus.rs と同一ファイルを検証 → TS/Rust の検出が byte 単位で一致。
  const corpus = JSON.parse(
    readFileSync(join(here, "..", "vectors", "year_corpus.json"), "utf8"),
  ) as { cases: { desc: string; text: string | null; expected: number | null }[] };
  assert.ok(corpus.cases.length > 0);
  for (const c of corpus.cases) {
    assert.equal(extractYear(c.text), c.expected, c.desc);
  }
});
