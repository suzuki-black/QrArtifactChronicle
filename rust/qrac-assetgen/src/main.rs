//! アセット生成ツール（docs/05 5.6 / docs/08 8.11）。
//! 出力:
//!   <out>/artifacts.sqlite                                  全rarityマス充足のベース遺物DB
//!   <out>/assets/base/<image_set_id>/<style>/0.png          ベース画像（被写体・透過・スタイル別）
//!   <out>/assets/layers/dirt/<id>.png                       汚れ（全遺物共有）
//!   <out>/assets/layers/damage/{wear,crack,chip}.png        破損（共有）
//!   <out>/assets/bg/<style>.png                             背景（スタイル別）
//! 最後に「全 civ×era×category × rarity 1..10」のカバレッジを検証し、欠けがあれば exit 1（CIゲート）。
//!
//! 現状ベース画像/レイヤーは手続き生成（qrac_render::art）。将来はアーティスト製アセットに差し替え。

use qrac_core::constants::{CATEGORIES, CIVILIZATIONS, DIRT_LAYERS, ERAS};
use qrac_render::art;
use qrac_render::ArtifactDb;
use std::fs;
use std::path::Path;
use std::process::exit;

const STYLES: [&str; 3] = ["museum", "dig", "catalog"];

fn write_png(pm: &tiny_skia::Pixmap, path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
    fs::write(path, pm.encode_png().expect("encode")).expect("write png");
}

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
                let pm = art::base_sprite(civ, cat); // 形状は civ,category に依存
                for style in STYLES {
                    // ベース被写体はスタイル間で共通（背景はスタイル別レイヤーで吸収）。
                    write_png(
                        &pm,
                        &assets
                            .join("base")
                            .join(image_set_id.to_string())
                            .join(style)
                            .join("0.png"),
                    );
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

    // 1b) 共有レイヤー（汚れ・破損）と背景（スタイル別）。
    let mut layer_count = 0usize;
    for (dirt, _) in DIRT_LAYERS.iter() {
        if let Some(pm) = art::dirt_texture(dirt) {
            write_png(
                &pm,
                &assets
                    .join("layers")
                    .join("dirt")
                    .join(format!("{dirt}.png")),
            );
            layer_count += 1;
        }
    }
    write_png(&art::wear_layer(), &assets.join("layers/damage/wear.png"));
    write_png(&art::crack_layer(), &assets.join("layers/damage/crack.png"));
    write_png(&art::chip_layer(), &assets.join("layers/damage/chip.png"));
    layer_count += 3;
    for style in STYLES {
        write_png(
            &art::background(style),
            &assets.join("bg").join(format!("{style}.png")),
        );
        layer_count += 1;
    }
    println!("  shared layers: {layer_count} files (dirt/damage/bg)");

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

    // 4) 出土の系譜: 参照グラフ生成（提案01 §3.1）。
    db.seed_references().expect("seed references");
    println!("  artifact_reference rows: {}", db.count_references());

    // 5) 到達性CIゲート（提案01 §4）: 全 to_set がランダムQRから現実的試行内で出現するか。
    let unreachable = db.verify_reference_reachability(50_000);
    if unreachable.is_empty() {
        println!("  ✅ reachability OK: 全参照先がサンプリングで到達可能");
    } else {
        eprintln!("  ❌ reachability FAILED: {} 件の到達不能", unreachable.len());
        for u in unreachable.iter().take(20) {
            eprintln!("     - {u}");
        }
        exit(1);
    }
    println!("== done ==");
}
