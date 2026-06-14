// ゴールデンベクタ生成。固定入力 → 期待出力を vectors/golden.json に書き出す。
// 移植先（他言語）はこの JSON を再現できなければならない（言語非依存の正本）。
// 使い方: npm run gen:vectors  （ロジック変更時のみ再生成し、差分をレビューする）

import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { deriveFromString } from "../src/derive.ts";
import { GEN_VERSION } from "../src/constants.ts";

const FIXED_INPUTS: { label: string; text: string; year?: number | null }[] = [
  { label: "empty", text: "" },
  { label: "ascii", text: "hello world" },
  { label: "url_plain", text: "https://example.com/path" },
  { label: "url_query", text: "https://EXAMPLE.com:443/Path/?b=2&a=1#frag" },
  { label: "japanese", text: "古代の遺物QRコード" },
  { label: "vcard_2014", text: "BEGIN:VCARD\nREV:2014-03-10T19:00:00Z\nEND:VCARD" },
  { label: "ical_2008", text: "BEGIN:VEVENT\nDTSTAMP:20081231T120000Z\nEND:VEVENT" },
  { label: "url_unix_2020", text: "https://t.example/x?t=1577836800" },
  { label: "iso_1985", text: "log entry 1985-06-15 archived" },
  { label: "year_override_1979", text: "anything", year: 1979 },
];

const vectors = FIXED_INPUTS.map((inp) => ({
  label: inp.label,
  // year は override のみ記録。auto抽出ケースは undefined → JSONから省かれ、再現時も auto抽出になる。
  input: { text: inp.text, ...(inp.year !== undefined ? { year: inp.year } : {}) },
  output: deriveFromString(inp.text, inp.year),
}));

const here = dirname(fileURLToPath(import.meta.url));
const out = join(here, "..", "vectors", "golden.json");
writeFileSync(out, JSON.stringify({ genVersion: GEN_VERSION, vectors }, null, 2) + "\n");
console.log(`wrote ${vectors.length} vectors → ${out}`);
