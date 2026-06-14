//! 🔒 QR正規化（docs/01 1.1）。reference/src/normalize.ts と1:1。
//! 汎用URLライブラリ非依存・%エンコード不可触・バイト精度。

const PRINTABLE_MIN_RATIO: f64 = 0.95;

pub fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

fn printable_ascii_ratio(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 1.0;
    }
    let mut ok = 0usize;
    for &b in bytes {
        if (0x20..=0x7e).contains(&b) || b == 0x09 || b == 0x0a || b == 0x0d {
            ok += 1;
        }
    }
    ok as f64 / bytes.len() as f64
}

pub fn is_probably_text(bytes: &[u8]) -> bool {
    is_valid_utf8(bytes) || printable_ascii_ratio(bytes) >= PRINTABLE_MIN_RATIO
}

/// 不正 UTF-8 は U+FFFD に置換して復号（lossy）。
pub fn decode_utf8_lenient(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// 末尾の空白系バイト（space/\t/\n/\r/\f/\v/NUL）のみ除去。
fn strip_trailing_whitespace(s: &str) -> &str {
    let ws = |b: u8| matches!(b, 0x20 | 0x09 | 0x0a | 0x0d | 0x0c | 0x0b | 0x00);
    let bytes = s.as_bytes();
    let mut end = bytes.len();
    while end > 0 && ws(bytes[end - 1]) {
        end -= 1;
    }
    &s[..end]
}

/// ASCII の A-Z のみ小文字化。非ASCII・記号は不変（バイト単位）。
fn ascii_lower_letters(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    for &b in s.as_bytes() {
        out.push(if (0x41..=0x5a).contains(&b) {
            b + 0x20
        } else {
            b
        });
    }
    // ASCII letter のみ変更するので UTF-8 妥当性は保たれる
    String::from_utf8(out).unwrap()
}

pub fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// URL のバイト精度正規化（docs/01 canonicalizeUrl）。
pub fn canonicalize_url(s: &str) -> String {
    // 1. 簡易自前パース
    let scheme_sep = s.find("://").unwrap();
    let scheme = ascii_lower_letters(&s[..scheme_sep]);
    let rest = &s[scheme_sep + 3..];

    let i = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..i];
    let mut remainder = &rest[i..];

    // fragment → query → path
    let mut fragment: Option<&str> = None;
    if let Some(h) = remainder.find('#') {
        fragment = Some(&remainder[h + 1..]);
        remainder = &remainder[..h];
    }
    let (mut path, query): (String, Option<&str>) = if let Some(q) = remainder.find('?') {
        (remainder[..q].to_string(), Some(&remainder[q + 1..]))
    } else {
        (remainder.to_string(), None)
    };

    // 2-4. authority = [userinfo '@'] host [':' port]
    let (userinfo, hostport): (&str, &str) = match authority.rfind('@') {
        Some(a) => (&authority[..a + 1], &authority[a + 1..]),
        None => ("", authority),
    };

    let host_raw;
    let mut port: Option<&str> = None;
    if hostport.starts_with('[') {
        // IPv6 リテラル: ']' までを host（括弧込み）、その後 ":port" があれば port
        let close = hostport.find(']').map(|c| c + 1).unwrap_or(hostport.len());
        host_raw = hostport[..close].to_string();
        if let Some(p) = hostport[close..].strip_prefix(':') {
            port = Some(p);
        }
    } else {
        match hostport.rfind(':') {
            Some(c)
                if !hostport[c + 1..].is_empty()
                    && hostport[c + 1..].bytes().all(|b| b.is_ascii_digit()) =>
            {
                host_raw = hostport[..c].to_string();
                port = Some(&hostport[c + 1..]);
            }
            _ => host_raw = hostport.to_string(),
        }
    }
    let host = ascii_lower_letters(&host_raw);

    // 4. 既定ポート除去
    if (scheme == "http" && port == Some("80")) || (scheme == "https" && port == Some("443")) {
        port = None;
    }

    // 5. path 末尾スラッシュ
    if path.is_empty() {
        path = "/".to_string();
    } else if path != "/" && path.ends_with('/') {
        path.pop();
    }

    // 6. query: キーの UTF-8 バイト辞書順で安定ソート（値・'=' 逐語保持）
    let query_out: Option<String> = match query {
        Some(q) if !q.is_empty() => {
            let mut parts: Vec<&str> = q.split('&').collect();
            parts.sort_by(|a, b| {
                let ka = a.split_once('=').map(|x| x.0).unwrap_or(a);
                let kb = b.split_once('=').map(|x| x.0).unwrap_or(b);
                ka.as_bytes().cmp(kb.as_bytes())
            });
            Some(parts.join("&"))
        }
        other => other.map(|s| s.to_string()),
    };

    // 8. 再構築
    let mut out = String::new();
    out.push_str(&scheme);
    out.push_str("://");
    out.push_str(userinfo);
    out.push_str(&host);
    if let Some(p) = port {
        out.push(':');
        out.push_str(p);
    }
    out.push_str(&path);
    if let Some(q) = query_out {
        out.push('?');
        out.push_str(&q);
    }
    if let Some(f) = fragment {
        out.push('#');
        out.push_str(f);
    }
    out
}

/// 正規化済みキー（ハッシュ入力）。
pub fn normalize_key(raw: &[u8]) -> Vec<u8> {
    if !is_probably_text(raw) {
        return raw.to_vec(); // バイナリは無加工
    }
    let decoded = decode_utf8_lenient(raw);
    let trimmed = strip_trailing_whitespace(&decoded);
    let s = if looks_like_url(trimmed) {
        canonicalize_url(trimmed)
    } else {
        trimmed.to_string()
    };
    s.into_bytes()
}
