//! ヒーロー画像生成: README 用の3点を docs/screenshots に書き出す。
use qrac_core::derive_from_string;
use qrac_core::hash::make_seed;
use qrac_core::normalize::normalize_key;
use qrac_render::{render_png, ArtifactDb};

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("usage: hero_gen <docs/screenshots dir>");
    let db = ArtifactDb::open_in_memory().unwrap();
    db.seed_full().unwrap();
    let picks = [
        ("hero-1.png", "https://museum.example/relic/28"), // mountain weapon (緑) mud chip+crack
        ("hero-2.png", "https://museum.example/relic/11"), // machine architecture (青) ★4 chip+crack+wear
        ("hero-3.png", "https://museum.example/relic/27"), // desert ritual (金) sand
    ];
    for (name, s) in picks {
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
        let path = format!("{out}/{name}");
        std::fs::write(&path, &png).unwrap();
        println!(
            "{path}  {} {} {}",
            attr.civ, attr.category, attr.dirt_layer_id
        );
    }
}
