//! 🔒 決定論コア: SHA-256・タグ付き派生RNG（docs/01 1.2/1.3）。
//! reference/src/hash.ts と1:1。big-endian 固定・タグNFC・整数しきい値（離散）/ f64（連続）。

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// tag を UTF-8 NFC 正規化（タグは ASCII なので実質 no-op だが規約として固定）。
fn nfc(tag: &str) -> String {
    tag.nfc().collect()
}

pub fn make_seed(normalized_key: &[u8]) -> [u8; 32] {
    sha256(normalized_key)
}

/// substream = SHA-256( seed || 0x00 || utf8(nfc(tag)) || u32be(counter) )
pub fn substream(seed: &[u8], tag: &str, counter: u32) -> [u8; 32] {
    let tag_n = nfc(tag);
    let mut v = Vec::with_capacity(seed.len() + 1 + tag_n.len() + 4);
    v.extend_from_slice(seed);
    v.push(0x00);
    v.extend_from_slice(tag_n.as_bytes());
    v.extend_from_slice(&counter.to_be_bytes());
    sha256(&v)
}

/// 一様実数 [0,1)。【連続出力専用】（colorMod / preservationScore）。
pub fn uniform(seed: &[u8], tag: &str) -> f64 {
    let s = substream(seed, tag, 0);
    let hi = u64::from_be_bytes(s[0..8].try_into().unwrap());
    hi as f64 / 2f64.powi(64)
}

/// 先頭4バイトの big-endian u32。【離散判定用】整数しきい値と直接比較。
pub fn u32v(seed: &[u8], tag: &str, counter: u32) -> u32 {
    let s = substream(seed, tag, counter);
    u32::from_be_bytes(s[0..4].try_into().unwrap())
}

/// 一様整数 [0, n)。剰余バイアス回避の棄却法。big-endian。
pub fn uint_below(seed: &[u8], tag: &str, n: u32) -> u32 {
    assert!(n > 0, "uint_below: n must be > 0");
    let n64 = n as u64;
    let limit = (0x1_0000_0000u64 / n64) * n64;
    let mut c: u32 = 0;
    loop {
        let x = u32v(seed, tag, c) as u64;
        if x < limit {
            return (x % n64) as u32;
        }
        c += 1;
    }
}

/// 重み付き選択（整数版・離散, docs/03 3.3 / docs/08 8.4）。
/// 値昇順に並べてから累積、r = uint_below(total)。重み0は選ばれない。
pub fn pick_weighted(seed: &[u8], tag: &str, items: &[(&str, u32)]) -> String {
    let mut sorted: Vec<(&str, u32)> = items.to_vec();
    sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes())); // 値の UTF-8 バイト昇順
    let total: u32 = sorted.iter().map(|i| i.1).sum();
    assert!(
        total > 0,
        "pick_weighted: total weight must be > 0 (tag={tag})"
    );
    let r = uint_below(seed, tag, total);
    let mut acc: u32 = 0;
    for (v, w) in &sorted {
        acc += *w;
        if r < acc {
            return v.to_string();
        }
    }
    sorted.last().unwrap().0.to_string()
}
