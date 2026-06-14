import { test } from "node:test";
import assert from "node:assert/strict";
import {
  normalizeKey,
  isProbablyText,
  looksLikeUrl,
  canonicalizeUrl,
} from "../src/normalize.ts";

const enc = (s: string) => new TextEncoder().encode(s);
const dec = (b: Uint8Array) => new TextDecoder().decode(b);

test("binary QR is passed through unchanged", () => {
  const bin = new Uint8Array([0x00, 0xff, 0xfe, 0x01, 0x80, 0x90, 0xab]);
  assert.deepEqual(normalizeKey(bin), bin);
  assert.equal(isProbablyText(bin), false);
});

test("text QR only strips trailing whitespace", () => {
  assert.equal(dec(normalizeKey(enc("hello \n\t"))), "hello");
  assert.equal(dec(normalizeKey(enc(" keep inner  spaces "))), " keep inner  spaces");
});

test("looksLikeUrl only matches http(s)://", () => {
  assert.ok(looksLikeUrl("http://x"));
  assert.ok(looksLikeUrl("https://x"));
  assert.equal(looksLikeUrl("ftp://x"), false);
  assert.equal(looksLikeUrl("mailto:a@b"), false);
  assert.equal(looksLikeUrl("HTTP://x"), false); // 先頭スキームは小文字でのみ判定（生のまま）
});

test("URL canonicalization: host lowercase, default port, trailing slash, query sort", () => {
  // ホスト小文字 + :443除去 + 末尾スラッシュ統一 + クエリキー順ソート、path大小とfragmentは保持
  const out = canonicalizeUrl("https://EXAMPLE.com:443/Path/?b=2&a=1#Frag");
  assert.equal(out, "https://example.com/Path?a=1&b=2#Frag");
});

test("URL canonicalization keeps query VALUES and path case as-is", () => {
  const out = canonicalizeUrl("https://h/AbC?k=ValUE");
  assert.equal(out, "https://h/AbC?k=ValUE");
});

test("normalizeKey: cosmetic URL variants collapse to same key", () => {
  const a = dec(normalizeKey(enc("https://Example.com/a?y=1&x=2")));
  const b = dec(normalizeKey(enc("https://example.com:443/a?x=2&y=1  ")));
  assert.equal(a, b);
});

test("normalizeKey: meaningful URL differences stay distinct", () => {
  const a = dec(normalizeKey(enc("https://h/Path")));
  const b = dec(normalizeKey(enc("https://h/path"))); // path大小は別物
  assert.notEqual(a, b);
});

// 以下は汎用URLライブラリと挙動が分かれる点（手実装の正本性を固定）
test("percent-encoding is left completely untouched", () => {
  // %2F を '/' に正規化しない、%xx の大文字小文字も変えない
  assert.equal(canonicalizeUrl("https://h/a%2Fb%3Fc"), "https://h/a%2Fb%3Fc");
  assert.equal(canonicalizeUrl("https://h/%e3%81%82?x=%2B"), "https://h/%e3%81%82?x=%2B");
});

test("non-default ports are preserved; only :80/:443 dropped", () => {
  assert.equal(canonicalizeUrl("http://h:8080/x"), "http://h:8080/x");
  assert.equal(canonicalizeUrl("http://h:80/x"), "http://h/x");
  assert.equal(canonicalizeUrl("https://h:443/x"), "https://h/x");
  assert.equal(canonicalizeUrl("https://h:80/x"), "https://h:80/x"); // 80 は https の既定でない→保持
});

test("userinfo and IPv6 host handled; host letters lowercased only", () => {
  assert.equal(canonicalizeUrl("https://User:Pw@HOST/x"), "https://User:Pw@host/x");
  assert.equal(canonicalizeUrl("http://[::1]:80/x"), "http://[::1]/x");
});

test("empty path becomes '/'; query without path keeps slash", () => {
  assert.equal(canonicalizeUrl("https://h"), "https://h/");
  assert.equal(canonicalizeUrl("https://h?b=1&a=2"), "https://h/?a=2&b=1");
});

test("query key sort is by UTF-8 bytes and stable for equal keys", () => {
  assert.equal(canonicalizeUrl("https://h/?b=1&a=2&a=1"), "https://h/?a=2&a=1&b=1");
});
