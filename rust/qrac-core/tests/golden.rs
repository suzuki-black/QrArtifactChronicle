//! TSオラクルが生成した golden.json を Rust が byte一致で再現することを検証（docs/08 8.9）。
//! これが緑であれば、決定論コアは TS/Rust で一致＝移植成功の中核条件を満たす。

use qrac_core::{derive_from_string, DerivedAttributes};
use serde_json::Value;

fn golden() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reference/vectors/golden.json"
    );
    let data = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&data).unwrap()
}

fn check(label: &str, got: &DerivedAttributes, exp: &Value) {
    assert_eq!(
        got.seed_hex,
        exp["seedHex"].as_str().unwrap(),
        "{label}: seedHex"
    );
    assert_eq!(
        got.artifact_hash,
        exp["artifactHash"].as_str().unwrap(),
        "{label}: artifactHash"
    );
    assert_eq!(
        got.base_rarity as u64,
        exp["baseRarity"].as_u64().unwrap(),
        "{label}: baseRarity"
    );
    assert_eq!(
        got.era_bonus as u64,
        exp["eraBonus"].as_u64().unwrap(),
        "{label}: eraBonus"
    );
    assert_eq!(
        got.final_rarity as u64,
        exp["finalRarity"].as_u64().unwrap(),
        "{label}: finalRarity"
    );
    assert_eq!(
        got.is_mythic,
        exp["isMythic"].as_bool().unwrap(),
        "{label}: isMythic"
    );
    assert_eq!(got.civ, exp["civ"].as_str().unwrap(), "{label}: civ");
    assert_eq!(got.era, exp["era"].as_str().unwrap(), "{label}: era");
    assert_eq!(
        got.category,
        exp["category"].as_str().unwrap(),
        "{label}: category"
    );
    // 連続値は f64 を厳密一致で比較（as_f64 は整数/浮動どちらの JSON 数値も f64 化）
    assert_eq!(
        got.color_mod.h_shift,
        exp["colorMod"]["hShift"].as_f64().unwrap(),
        "{label}: hShift"
    );
    assert_eq!(
        got.color_mod.s_mul,
        exp["colorMod"]["sMul"].as_f64().unwrap(),
        "{label}: sMul"
    );
    assert_eq!(
        got.color_mod.v_mul,
        exp["colorMod"]["vMul"].as_f64().unwrap(),
        "{label}: vMul"
    );
    assert_eq!(
        got.dirt_layer_id,
        exp["dirtLayerId"].as_str().unwrap(),
        "{label}: dirtLayerId"
    );
    assert_eq!(
        got.damage.chip,
        exp["damage"]["chip"].as_bool().unwrap(),
        "{label}: chip"
    );
    assert_eq!(
        got.damage.crack,
        exp["damage"]["crack"].as_bool().unwrap(),
        "{label}: crack"
    );
    assert_eq!(
        got.damage.wear,
        exp["damage"]["wear"].as_bool().unwrap(),
        "{label}: wear"
    );
    assert_eq!(
        got.preservation_score,
        exp["preservationScore"].as_f64().unwrap(),
        "{label}: preservationScore"
    );
}

#[test]
fn golden_vectors_reproduce_exactly() {
    let g = golden();
    let vectors = g["vectors"].as_array().expect("vectors array");
    assert!(!vectors.is_empty());
    for v in vectors {
        let label = v["label"].as_str().unwrap();
        let text = v["input"]["text"].as_str().unwrap();
        // year: キーがあれば override（数値→Some(Some) / null→Some(None)）、無ければ自動抽出(None)
        let year_override: Option<Option<i32>> = match v["input"].get("year") {
            Some(yv) => Some(yv.as_i64().map(|x| x as i32)),
            None => None,
        };
        let got = derive_from_string(text, year_override);
        check(label, &got, &v["output"]);
    }
}
