//! アセット生成ツール（docs/05 5.6 / docs/08 8.11）。
//! 出力:
//!   <out>/artifacts.sqlite                                  全rarityマス充足のベース遺物DB
//!   <out>/assets/base/<image_set_id>/<style>/0.png          ベース画像（被写体・透過・ニュートラル）
//! 最後に「全 civ×era×category × rarity 1..10」のカバレッジを検証し、欠けがあれば exit 1（CIゲート）。
//!
//! Step1 ではベース画像は手続き生成のスプライト。将来はアーティスト製アセットに差し替え。

use qrac_core::constants::{CATEGORIES, CIVILIZATIONS, ERAS};
use qrac_render::{render_base_sprite, ArtifactDb};
use std::fs;
use std::path::Path;
use std::process::exit;

const STYLES: [&str; 3] = ["museum", "dig", "catalog"];

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist".to_string());
    let out = Path::new(&out);
    let assets = out.join("assets");
    let db_path = out.join("artifacts.sqlite");

    println!("== qrac-assetgen → {} ==", out.display());
    let _ = fs::remove_dir_all(&assets);
    let _ = fs::remove_file(&db_path);
    fs::create_dir_all(&assets).expect("mkdir assets");

    // 1) ベース画像生成（image_set_id = ci*100+ei*10+ki, seed_full と一致）。
    let mut sprite_count = 0usize;
    for (ci, (civ, _)) in CIVILIZATIONS.iter().enumerate() {
        for (ei, (_era, _)) in ERAS.iter().enumerate() {
            for (ki, (cat, _)) in CATEGORIES.iter().enumerate() {
                let image_set_id = ci * 100 + ei * 10 + ki;
                let pm = render_base_sprite(civ, cat); // 形状は civ,category に依存
                for style in STYLES {
                    let dir = assets
                        .join("base")
                        .join(image_set_id.to_string())
                        .join(style);
                    fs::create_dir_all(&dir).expect("mkdir base");
                    // 実運用は WebP。ここでは PNG（tiny-skia 標準エンコード）。
                    let png = pm.encode_png().expect("encode");
                    fs::write(dir.join("0.png"), &png).expect("write png");
                    sprite_count += 1;
                }
            }
        }
    }
    println!(
        "  base sprites: {sprite_count} files ({} image_set_id × {} styles)",
        sprite_count / STYLES.len(),
        STYLES.len()
    );

    // 2) DB（全rarityマス充足）。
    let db = ArtifactDb::open(db_path.to_str().unwrap()).expect("open db");
    db.seed_full().expect("seed");
    println!("  base_artifact rows: {}", db.count_non_global());

    // 3) カバレッジゲート（docs/05 5.6）。
    let gaps = db.verify_full_coverage();
    if gaps.is_empty() {
        let combos = CIVILIZATIONS.len() * ERAS.len() * CATEGORIES.len();
        println!("  ✅ coverage OK: 全 {combos} combo (civ×era×category) で ★1..10 充足");
    } else {
        eprintln!("  ❌ coverage FAILED: {} 件の欠け", gaps.len());
        for g in gaps.iter().take(20) {
            eprintln!("     - {g}");
        }
        exit(1);
    }
    println!("== done ==");
}
