//! Rust 側のプロパティ/境界テスト（reference/test/* と対。docs/08 8.9）。
//! golden に含まれない負例・分布・独立性・正規化のバイト精度を固定する。

use qrac_core::appearance::color_mod;
use qrac_core::hash::{make_seed, pick_weighted};
use qrac_core::normalize::{canonicalize_url, is_probably_text, looks_like_url, normalize_key};
use qrac_core::rarity::{base_rarity, era_bonus_for, final_rarity, is_mythic};
use qrac_core::timestamp::extract_year;

// --- timestamp ---

#[test]
fn extract_year_explicit_only_newest_wins() {
    assert_eq!(
        extract_year(Some("BEGIN:VCARD\nREV:2014-03-10T00:00:00Z")),
        Some(2014)
    );
    assert_eq!(extract_year(Some("DTSTAMP:20081231T120000Z")), Some(2008));
    assert_eq!(extract_year(Some("https://t/x?t=1577836800")), Some(2020));
    assert_eq!(extract_year(Some("note 1985-06-15")), Some(1985));
    assert_eq!(extract_year(Some("1985-01-01 and 2003-05-05")), Some(2003)); // 最も新しい
                                                                             // 日時でない数値は拾わない
    assert_eq!(extract_year(Some("tel:+81-3-1234-5678")), None);
    assert_eq!(extract_year(Some("price 1980 yen only")), None);
    assert_eq!(extract_year(Some("plain text no date")), None);
    assert_eq!(extract_year(None), None);
}

// --- rarity ---

#[test]
fn era_bonus_boundaries_and_invalid() {
    assert_eq!(era_bonus_for(Some(2025)), 0);
    assert_eq!(era_bonus_for(Some(2019)), 1);
    assert_eq!(era_bonus_for(Some(2000)), 2);
    assert_eq!(era_bonus_for(Some(1990)), 3);
    assert_eq!(era_bonus_for(Some(1980)), 4);
    assert_eq!(era_bonus_for(Some(1979)), 5);
    assert_eq!(era_bonus_for(None), 0);
    assert_eq!(era_bonus_for(Some(1899)), 0); // 不正
    assert_eq!(era_bonus_for(Some(2050)), 0); // 未来
}

#[test]
fn clamp_and_mythic_property() {
    assert_eq!(final_rarity(10, 5), 13);
    assert_eq!(final_rarity(9, 3), 12);
    for base in 1..=10 {
        assert!(!is_mythic(final_rarity(base, 0))); // bonus0 では神話級不可
    }
    assert!(is_mythic(final_rarity(10, 1)));
}

// --- normalize（バイト精度） ---

#[test]
fn binary_passthrough_and_text_detect() {
    let bin = [0x00u8, 0xff, 0xfe, 0x01, 0x80, 0x90, 0xab];
    assert!(!is_probably_text(&bin));
    assert_eq!(normalize_key(&bin), bin.to_vec());
}

#[test]
fn url_canonicalization_matches_spec() {
    assert!(looks_like_url("https://x"));
    assert!(!looks_like_url("ftp://x"));
    assert_eq!(
        canonicalize_url("https://EXAMPLE.com:443/Path/?b=2&a=1#Frag"),
        "https://example.com/Path?a=1&b=2#Frag"
    );
    // %エンコード不可触
    assert_eq!(
        canonicalize_url("https://h/a%2Fb%3Fc"),
        "https://h/a%2Fb%3Fc"
    );
    // 非既定ポート保持 / 既定のみ除去
    assert_eq!(canonicalize_url("http://h:8080/x"), "http://h:8080/x");
    assert_eq!(canonicalize_url("http://h:80/x"), "http://h/x");
    assert_eq!(canonicalize_url("https://h:80/x"), "https://h:80/x");
    // userinfo / IPv6
    assert_eq!(
        canonicalize_url("https://User:Pw@HOST/x"),
        "https://User:Pw@host/x"
    );
    assert_eq!(canonicalize_url("http://[::1]:80/x"), "http://[::1]/x");
    // 空パス→/ ・キーのバイト順安定ソート
    assert_eq!(canonicalize_url("https://h?b=1&a=2"), "https://h/?a=2&b=1");
    assert_eq!(
        canonicalize_url("https://h/?b=1&a=2&a=1"),
        "https://h/?a=2&a=1&b=1"
    );
}

#[test]
fn cosmetic_url_variants_collapse() {
    let a = normalize_key(b"https://Example.com/a?y=1&x=2");
    let b = normalize_key(b"https://example.com:443/a?x=2&y=1  ");
    assert_eq!(a, b);
    // path 大小は別物
    assert_ne!(
        normalize_key(b"https://h/Path"),
        normalize_key(b"https://h/path")
    );
}

// --- hash / pick_weighted ---

#[test]
fn pick_weighted_skips_zero_and_is_order_independent() {
    let items = [("z", 0u32), ("a", 1), ("m", 1)];
    let shuffled = [("m", 1u32), ("z", 0), ("a", 1)];
    for i in 0..500 {
        let seed = make_seed(format!("s{i}").as_bytes());
        let x = pick_weighted(&seed, "t", &items);
        assert_ne!(x, "z");
        assert_eq!(x, pick_weighted(&seed, "t", &shuffled));
    }
}

// --- 分布・独立性 ---

#[test]
fn rarity_distribution_converges() {
    let n = 300_000u32;
    let mut counts = [0u64; 11];
    for i in 0..n {
        let seed = make_seed(format!("seed#{i}").as_bytes());
        counts[base_rarity(&seed) as usize] += 1;
    }
    let p1 = counts[1] as f64 / n as f64;
    assert!((p1 - 0.4049).abs() < 0.006, "★1 = {p1}");
    let p2 = counts[2] as f64 / n as f64;
    assert!((p2 - 0.25).abs() < 0.006, "★2 = {p2}");
    assert_eq!(counts[0], 0);
}

#[test]
fn rarity_and_color_uncorrelated() {
    let n = 40_000usize;
    let (mut rar, mut hue) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for i in 0..n {
        let seed = make_seed(format!("ind#{i}").as_bytes());
        rar.push(base_rarity(&seed) as f64);
        hue.push(color_mod(&seed).h_shift);
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (mr, mh) = (mean(&rar), mean(&hue));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (dx, dy) = (rar[i] - mr, hue[i] - mh);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    let r = sxy / (sxx * syy).sqrt();
    assert!(r.abs() < 0.02, "corr(rarity,hShift) = {r}");
}
