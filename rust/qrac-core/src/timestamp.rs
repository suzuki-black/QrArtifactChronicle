//! QR内容からの生成年抽出（docs/02 2.2 / docs/07 Q1）。
//! reference/src/timestamp.ts と同値。明示的日時のみ・推測しない・最も新しい年を採用。
//! 正規表現は使わず手スキャン（lookaround挙動の言語差を避けるため）。

const PLAUSIBLE_MIN: i32 = 1970;
const PLAUSIBLE_MAX: i32 = 2099;

fn plausible(y: i32) -> bool {
    (PLAUSIBLE_MIN..=PLAUSIBLE_MAX).contains(&y)
}

/// epoch ms → UTC年（Hinnant civil_from_days）。chrono非依存。
fn year_from_unix_ms(ms: i64) -> i32 {
    let days = ms.div_euclid(86_400_000);
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    (if mp >= 10 { y + 1 } else { y }) as i32
}

fn year_from_unix(n: i64) -> Option<i32> {
    let ms = if n < 1_000_000_000_000 { n * 1000 } else { n };
    let y = year_from_unix_ms(ms);
    if plausible(y) {
        Some(y)
    } else {
        None
    }
}

/// vCard/iCal の body 内で最初の (\d{4})(\d{2}|-\d{2}) を探し、その4桁を返す。
fn find_year_in_body(body: &str) -> Option<i32> {
    let b = body.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i + 4 <= n {
        if b[i].is_ascii_digit()
            && b[i + 1].is_ascii_digit()
            && b[i + 2].is_ascii_digit()
            && b[i + 3].is_ascii_digit()
        {
            let two_digits = i + 6 <= n && b[i + 4].is_ascii_digit() && b[i + 5].is_ascii_digit();
            let dash_two = i + 7 <= n
                && b[i + 4] == b'-'
                && b[i + 5].is_ascii_digit()
                && b[i + 6].is_ascii_digit();
            if two_digits || dash_two {
                return std::str::from_utf8(&b[i..i + 4]).ok()?.parse().ok();
            }
        }
        i += 1;
    }
    None
}

pub fn extract_year(text: Option<&str>) -> Option<i32> {
    let text = text?;
    let mut candidates: Vec<i32> = Vec::new();

    // 1. vCard REV / iCal DTSTAMP・DTSTART（行頭ラベル直後）
    for line in text.split('\n') {
        let trimmed = line.trim_start_matches([' ', '\t', '\r']);
        let upper = trimmed.to_ascii_uppercase();
        for label in ["REV", "DTSTAMP", "DTSTART"] {
            if upper.starts_with(label) {
                let after = &trimmed[label.len()..];
                let mut chars = after.chars();
                if matches!(chars.next(), Some(':') | Some(';')) {
                    if let Some(y) = find_year_in_body(&after[1..]) {
                        if plausible(y) {
                            candidates.push(y);
                        }
                    }
                }
            }
        }
    }

    // 2. URL内の UNIX タイムスタンプ（10/13桁の独立数値＝極大桁run）
    if text.starts_with("http://") || text.starts_with("https://") {
        let b = text.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i].is_ascii_digit() {
                let start = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                let len = i - start;
                if len == 10 || len == 13 {
                    if let Ok(num) = std::str::from_utf8(&b[start..i]).unwrap().parse::<i64>() {
                        if let Some(y) = year_from_unix(num) {
                            candidates.push(y);
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    // 3. ISO 8601 日付 YYYY-MM-DD（前後が数字でない・月日妥当）
    let b = text.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i + 10 <= n {
        let boundary_before = i == 0 || !b[i - 1].is_ascii_digit();
        let shape = b[i..i + 4].iter().all(|c| c.is_ascii_digit())
            && b[i + 4] == b'-'
            && b[i + 5].is_ascii_digit()
            && b[i + 6].is_ascii_digit()
            && b[i + 7] == b'-'
            && b[i + 8].is_ascii_digit()
            && b[i + 9].is_ascii_digit();
        let boundary_after = i + 10 >= n || !b[i + 10].is_ascii_digit();
        if boundary_before && shape && boundary_after {
            let y: i32 = std::str::from_utf8(&b[i..i + 4]).unwrap().parse().unwrap();
            let mo: i32 = std::str::from_utf8(&b[i + 5..i + 7])
                .unwrap()
                .parse()
                .unwrap();
            let d: i32 = std::str::from_utf8(&b[i + 8..i + 10])
                .unwrap()
                .parse()
                .unwrap();
            if plausible(y) && (1..=12).contains(&mo) && (1..=31).contains(&d) {
                candidates.push(y);
            }
            i += 10;
            continue;
        }
        i += 1;
    }

    candidates.into_iter().max()
}
