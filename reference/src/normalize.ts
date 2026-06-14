// 🔒 QR正規化（docs/01 1.1 / docs/00 0.4.5）。
// 手実装。汎用URLライブラリ（JSの URL クラス等）は使わない。
// この実装は Rust 本実装とバイト一致させる「正本」。docs/01 canonicalizeUrl と1:1対応。

const PRINTABLE_MIN_RATIO = 0.95;
const enc = new TextEncoder();
const utf8 = (s: string): Uint8Array => enc.encode(s);

// --- テキスト/バイナリ判定 ---

export function isValidUtf8(bytes: Uint8Array): boolean {
  try {
    new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    return true;
  } catch {
    return false;
  }
}

function printableAsciiRatio(bytes: Uint8Array): number {
  if (bytes.length === 0) return 1;
  let ok = 0;
  for (const b of bytes) {
    if ((b >= 0x20 && b <= 0x7e) || b === 0x09 || b === 0x0a || b === 0x0d) ok++;
  }
  return ok / bytes.length;
}

/** テキストQRかバイナリQRか（docs/01 ヘルパ定義）。 */
export function isProbablyText(bytes: Uint8Array): boolean {
  return isValidUtf8(bytes) || printableAsciiRatio(bytes) >= PRINTABLE_MIN_RATIO;
}

/** 不正 UTF-8 は U+FFFD に置換して復号（lenient）。 */
export function decodeUtf8Lenient(bytes: Uint8Array): string {
  return new TextDecoder("utf-8", { fatal: false }).decode(bytes);
}

// --- 文字列ヘルパ（バイト精度・ロケール非依存） ---

/** 末尾の空白系バイト（space/\t/\n/\r/\f/\v/NUL）のみ除去。\s は使わない（範囲が曖昧なため）。 */
function stripTrailingWhitespace(s: string): string {
  const ws = new Set([0x20, 0x09, 0x0a, 0x0d, 0x0c, 0x0b, 0x00]);
  let end = s.length;
  while (end > 0 && ws.has(s.charCodeAt(end - 1))) end--;
  return s.slice(0, end);
}

/** ASCII の A-Z のみ小文字化。非ASCII・記号は不変。 */
function asciiLowerLetters(s: string): string {
  let out = "";
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    out += c >= 0x41 && c <= 0x5a ? String.fromCharCode(c + 0x20) : s[i];
  }
  return out;
}

/** UTF-8 バイト辞書順比較。 */
function compareBytes(a: Uint8Array, b: Uint8Array): number {
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i++) if (a[i] !== b[i]) return a[i]! - b[i]!;
  return a.length - b.length;
}

/** http:// または https:// で始まる場合のみ URL とみなす（docs/01）。 */
export function looksLikeUrl(s: string): boolean {
  return s.startsWith("http://") || s.startsWith("https://");
}

/**
 * URL のバイト精度正規化（docs/01 canonicalizeUrl）。汎用ライブラリ非依存・%エンコード不可触。
 */
export function canonicalizeUrl(s: string): string {
  // 1. 簡易自前パース
  const schemeSep = s.indexOf("://");
  const scheme = asciiLowerLetters(s.slice(0, schemeSep));
  const rest = s.slice(schemeSep + 3);

  // authority 終端 = 最初の '/' '?' '#'
  let i = 0;
  while (i < rest.length && rest[i] !== "/" && rest[i] !== "?" && rest[i] !== "#") i++;
  const authority = rest.slice(0, i);
  let remainder = rest.slice(i);

  // fragment → query → path の順に切り出す（デコードしない）
  let fragment: string | null = null;
  const hashIdx = remainder.indexOf("#");
  if (hashIdx >= 0) {
    fragment = remainder.slice(hashIdx + 1);
    remainder = remainder.slice(0, hashIdx);
  }
  let query: string | null = null;
  let path: string;
  const qIdx = remainder.indexOf("?");
  if (qIdx >= 0) {
    query = remainder.slice(qIdx + 1);
    path = remainder.slice(0, qIdx);
  } else {
    path = remainder;
  }

  // 2-4. authority = [userinfo '@'] host [':' port]
  const atIdx = authority.lastIndexOf("@");
  const userinfo = atIdx >= 0 ? authority.slice(0, atIdx + 1) : ""; // '@' 込みで逐語保持
  const hostport = atIdx >= 0 ? authority.slice(atIdx + 1) : authority;

  let host: string;
  let port: string | null = null;
  if (hostport.startsWith("[")) {
    // IPv6 リテラル
    const close = hostport.indexOf("]");
    host = hostport.slice(0, close + 1);
    const after = hostport.slice(close + 1);
    if (after.startsWith(":")) port = after.slice(1);
  } else {
    const colon = hostport.lastIndexOf(":");
    if (colon >= 0 && /^[0-9]+$/.test(hostport.slice(colon + 1))) {
      host = hostport.slice(0, colon);
      port = hostport.slice(colon + 1);
    } else {
      host = hostport;
    }
  }
  host = asciiLowerLetters(host); // ASCII A-Z のみ小文字化（IDN変換しない）

  // 4. 既定ポート除去
  if ((scheme === "http" && port === "80") || (scheme === "https" && port === "443")) {
    port = null;
  }

  // 5. path 末尾スラッシュ（'/' 単独は維持、それ以外の末尾 '/' を1つ除去、空は '/'）
  if (path === "") path = "/";
  else if (path !== "/" && path.endsWith("/")) path = path.slice(0, -1);

  // 6. query: キーの UTF-8 バイト辞書順で安定ソート（値・'=' は逐語保持）
  if (query !== null && query.length > 0) {
    const parts = query.split("&");
    const keyed = parts.map((p, idx) => {
      const eq = p.indexOf("=");
      const key = eq >= 0 ? p.slice(0, eq) : p;
      return { p, key: utf8(key), idx };
    });
    keyed.sort((a, b) => compareBytes(a.key, b.key) || a.idx - b.idx); // 安定（idxタイブレーク）
    query = keyed.map((k) => k.p).join("&");
  }

  // 8. 再構築
  let out = `${scheme}://${userinfo}${host}${port !== null ? ":" + port : ""}${path}`;
  if (query !== null) out += "?" + query;
  if (fragment !== null) out += "#" + fragment;
  return out;
}

/**
 * 正規化済みキー（ハッシュ入力）。
 * バイナリはそのまま、テキストは末尾整形、URLのみ手実装正規化。
 */
export function normalizeKey(rawBytes: Uint8Array): Uint8Array {
  if (!isProbablyText(rawBytes)) return rawBytes; // バイナリは無加工

  let s = decodeUtf8Lenient(rawBytes);
  s = stripTrailingWhitespace(s);
  if (looksLikeUrl(s)) s = canonicalizeUrl(s);
  return utf8(s);
}
