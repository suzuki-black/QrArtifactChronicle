//! extractYear 回帰コーパス（docs/07 Q1）。TSオラクルと共有する year_corpus.json を読み、
//! Rust の extract_year が `expected` を厳密再現することを検証する（TS側は rarity.test.ts が同ファイルを検証）。

use qrac_core::timestamp::extract_year;
use serde_json::Value;

fn corpus() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reference/vectors/year_corpus.json"
    );
    let data = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&data).unwrap()
}

#[test]
fn extract_year_matches_shared_corpus() {
    let c = corpus();
    let cases = c["cases"].as_array().expect("cases array");
    assert!(!cases.is_empty());
    for case in cases {
        let desc = case["desc"].as_str().unwrap();
        let text: Option<&str> = case["text"].as_str(); // null → None
        let expected: Option<i32> = case["expected"].as_i64().map(|x| x as i32); // null → None
        assert_eq!(extract_year(text), expected, "case: {desc}");
    }
}
