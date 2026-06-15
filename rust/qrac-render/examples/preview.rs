//! 手動確認用プレビュー: いくつかのQR文字列を6層合成して PNG 出力する。
//! 使い方: cargo run -p qrac-render --example preview -- <out_dir>
use qrac_core::derive_from_string;
use qrac_core::hash::make_seed;
use qrac_core::normalize::normalize_key;
use qrac_render::{render_png, ArtifactDb};

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/qrac_preview".into());
    std::fs::create_dir_all(&out).unwrap();
    let db = ArtifactDb::open_in_memory().unwrap();
    db.seed_full().unwrap();

    let samples = [
        "https://example.com/welcome",
        "https://anthropic.com",
        "WIFI:S:MyNet;T:WPA;P:secret;;",
        "tel:+81-3-1234-5678",
        "https://shrine.jp/omikuji?year=1923",
        "ぐるぐる古代文明",
        "https://github.com/suzuki-black/QrArtifactChronicle",
        "0123456789",
    ];
    for (i, s) in samples.iter().enumerate() {
        let attr = derive_from_string(s, None);
        let seed = make_seed(&normalize_key(s.as_bytes()));
        let base = db.select_base(
            &seed,
            &attr.civ,
            &attr.era,
            &attr.category,
            attr.base_rarity,
        );
        let png = render_png(&attr, &base, None);
        let path = format!(
            "{out}/{i}_{}_{}_{}.png",
            attr.civ, attr.category, attr.dirt_layer_id
        );
        std::fs::write(&path, &png).unwrap();
        println!(
            "{path}  ★{} {} dirt={} chip={} crack={} wear={} preserve={:.2}",
            attr.final_rarity,
            attr.era,
            attr.dirt_layer_id,
            attr.damage.chip,
            attr.damage.crack,
            attr.damage.wear,
            attr.preservation_score
        );
    }
}
