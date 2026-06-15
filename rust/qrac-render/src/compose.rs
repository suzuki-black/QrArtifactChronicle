//! 画像合成（docs/05）。tiny-skia CPU。6層を固定順で合成（docs/05 5.2 / docs/00 0.4.3）。
//! 合成順序: baseImage → HSV → preservation → (chip) → background合成 → dirt → damage。
//! レイヤーはアセット(PNG)から読み込み、無ければ `art` モジュールで手続き生成にフォールバック。
//! ピクセル厳密一致は決定論の必須要件ではない（同一性は artifactHash／属性, docs/08 8.6）。

use crate::art;
use crate::db::BaseArtifact;
use qrac_core::types::{ColorMod, DerivedAttributes};
use std::path::Path;
use tiny_skia::{BlendMode, Mask, MaskType, Pixmap, PixmapPaint, Transform};

pub use crate::art::{CANVAS_H, CANVAS_W};

const STYLE: &str = "museum";

fn load_png(path: &Path) -> Option<Pixmap> {
    std::fs::read(path)
        .ok()
        .and_then(|b| Pixmap::decode_png(&b).ok())
}

/// 層2: HSV補正（透明画素は不変）。連続出力。
fn apply_hsv(pm: &mut Pixmap, cm: &ColorMod) {
    for px in pm.pixels_mut() {
        let c = px.demultiply();
        if c.alpha() == 0 {
            continue;
        }
        let (h, s, v) = art::rgb_to_hsv(c.red(), c.green(), c.blue());
        let (r, g, b) = art::hsv_to_rgb(
            h + cm.h_shift as f32,
            (s * cm.s_mul as f32).clamp(0.0, 1.0),
            (v * cm.v_mul as f32).clamp(0.0, 1.0),
        );
        *px = tiny_skia::ColorU8::from_rgba(r, g, b, c.alpha()).premultiply();
    }
}

/// 層5: 保存状態のカラーグレード（彩度/明度/コントラスト）。score 0=劣悪..1=良好。
fn apply_preservation(pm: &mut Pixmap, score: f64) {
    let s = score as f32;
    let sat = 0.70 + 0.40 * s; // 0.70..1.10
    let bri = 0.92 + 0.13 * s; // 0.92..1.05
    let con = 0.90 + 0.18 * s; // 0.90..1.08
    for px in pm.pixels_mut() {
        let c = px.demultiply();
        if c.alpha() == 0 {
            continue;
        }
        let (h, st, v) = art::rgb_to_hsv(c.red(), c.green(), c.blue());
        let nv = (((v - 0.5) * con + 0.5) * bri).clamp(0.0, 1.0);
        let (r, g, b) = art::hsv_to_rgb(h, (st * sat).clamp(0.0, 1.0), nv);
        *px = tiny_skia::ColorU8::from_rgba(r, g, b, c.alpha()).premultiply();
    }
}

fn dirt_blend(dirt: &str) -> BlendMode {
    match dirt {
        "sea_salt" => BlendMode::Screen, // 白い析出は明るく
        _ => BlendMode::Multiply,        // 泥/砂/煤/灰は暗く沈める
    }
}
fn dirt_strength(s: f64) -> f32 {
    (0.9 - 0.7 * s as f32).clamp(0.15, 0.9)
}
fn damage_strength(s: f64) -> f32 {
    (0.9 - 0.6 * s as f32).clamp(0.2, 0.9)
}

/// artifact_hash 由来の小さな回転角（汚れテクスチャに個体差を与える）。
fn hash_angle(hash: &str) -> f32 {
    let n = u32::from_str_radix(&hash.chars().take(4).collect::<String>(), 16).unwrap_or(0);
    (n % 360) as f32
}

/// 6層を合成して Pixmap を返す。`assets_dir` 配下のPNGを使い、無ければ手続き生成。
pub fn render(attr: &DerivedAttributes, base: &BaseArtifact, assets_dir: Option<&Path>) -> Pixmap {
    let id = Transform::identity();

    // 層1: ベース被写体（アセット or 手続き）
    let base_png = assets_dir.and_then(|d| {
        load_png(
            &d.join("base")
                .join(base.image_set_id.to_string())
                .join(format!("{STYLE}/0.png")),
        )
    });
    let mut subject = base_png.unwrap_or_else(|| art::base_sprite(&base.civ, &base.category));

    // 層2: HSV / 層5: 保存状態（被写体に直接）
    apply_hsv(&mut subject, &attr.color_mod);
    apply_preservation(&mut subject, attr.preservation_score);

    // 欠け: 被写体のアルファを削る（DestinationOut）
    if attr.damage.chip {
        let chip = load_layer(assets_dir, "layers/damage/chip.png", art::chip_layer);
        subject.draw_pixmap(
            0,
            0,
            chip.as_ref(),
            &PixmapPaint {
                blend_mode: BlendMode::DestinationOut,
                ..Default::default()
            },
            id,
            None,
        );
    }

    // 被写体のシルエットマスク（汚れ/破損を被写体内に限定）
    let mask = Mask::from_pixmap(subject.as_ref(), MaskType::Alpha);

    // 層6: 背景 → その上に被写体
    let mut canvas = load_layer(assets_dir, &format!("bg/{STYLE}.png"), || {
        art::background(STYLE)
    });
    canvas.draw_pixmap(0, 0, subject.as_ref(), &PixmapPaint::default(), id, None);

    // 層3: 汚れ（マスクで被写体内に限定、個体差で回転）
    if attr.dirt_layer_id != "none" {
        if let Some(dirt) = load_layer_opt(assets_dir, &attr.dirt_layer_id) {
            let rot = Transform::from_rotate_at(hash_angle(&attr.artifact_hash), 256.0, 256.0);
            canvas.draw_pixmap(
                0,
                0,
                dirt.as_ref(),
                &PixmapPaint {
                    blend_mode: dirt_blend(&attr.dirt_layer_id),
                    opacity: dirt_strength(attr.preservation_score),
                    ..Default::default()
                },
                rot,
                Some(&mask),
            );
        }
    }

    // 層4: 摩耗 / ひび
    let ds = damage_strength(attr.preservation_score);
    if attr.damage.wear {
        let wear = load_layer(assets_dir, "layers/damage/wear.png", art::wear_layer);
        canvas.draw_pixmap(
            0,
            0,
            wear.as_ref(),
            &PixmapPaint {
                blend_mode: BlendMode::SoftLight,
                opacity: ds,
                ..Default::default()
            },
            id,
            Some(&mask),
        );
    }
    if attr.damage.crack {
        let crack = load_layer(assets_dir, "layers/damage/crack.png", art::crack_layer);
        canvas.draw_pixmap(
            0,
            0,
            crack.as_ref(),
            &PixmapPaint {
                blend_mode: BlendMode::Multiply,
                opacity: ds,
                ..Default::default()
            },
            id,
            Some(&mask),
        );
    }
    canvas
}

fn load_layer<F: FnOnce() -> Pixmap>(dir: Option<&Path>, rel: &str, fallback: F) -> Pixmap {
    dir.and_then(|d| load_png(&d.join(rel)))
        .unwrap_or_else(fallback)
}

/// 汚れ: アセット優先、無ければ手続き生成（none は None）。
fn load_layer_opt(dir: Option<&Path>, dirt: &str) -> Option<Pixmap> {
    if let Some(p) = dir.and_then(|d| load_png(&d.join(format!("layers/dirt/{dirt}.png")))) {
        return Some(p);
    }
    art::dirt_texture(dirt)
}

/// 直 RGBA8（非プリマルチプライ）。
pub struct RgbaBuf {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn render_rgba(
    attr: &DerivedAttributes,
    base: &BaseArtifact,
    assets_dir: Option<&Path>,
) -> RgbaBuf {
    let pm = render(attr, base, assets_dir);
    let mut rgba = Vec::with_capacity((CANVAS_W * CANVAS_H * 4) as usize);
    for px in pm.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaBuf {
        width: CANVAS_W,
        height: CANVAS_H,
        rgba,
    }
}

pub fn render_png(
    attr: &DerivedAttributes,
    base: &BaseArtifact,
    assets_dir: Option<&Path>,
) -> Vec<u8> {
    render(attr, base, assets_dir)
        .encode_png()
        .expect("encode_png")
}

/// ベーススプライト（互換: assetgen 等で利用）。
pub fn render_base_sprite(civ: &str, category: &str) -> Pixmap {
    art::base_sprite(civ, category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ArtifactDb;
    use qrac_core::derive_from_string;
    use qrac_core::hash::make_seed;
    use qrac_core::normalize::normalize_key;

    fn setup() -> (DerivedAttributes, BaseArtifact) {
        let db = ArtifactDb::open_in_memory().unwrap();
        db.seed_full().unwrap();
        let attr = derive_from_string("https://example.com/welcome", None);
        let seed = make_seed(&normalize_key(b"https://example.com/welcome"));
        let base = db.select_base(
            &seed,
            &attr.civ,
            &attr.era,
            &attr.category,
            attr.base_rarity,
        );
        (attr, base)
    }

    #[test]
    fn renders_nonblank_and_deterministic() {
        let (attr, base) = setup();
        let png1 = render_png(&attr, &base, None);
        let png2 = render_png(&attr, &base, None);
        assert!(png1.len() > 1000);
        assert_eq!(png1, png2, "render must be deterministic");
        // 中心は被写体内なので背景のみではない
        let buf = render_rgba(&attr, &base, None);
        let ci = ((256 * CANVAS_W + 256) * 4) as usize;
        assert!(
            buf.rgba[ci + 3] == 255,
            "center should be opaque (subject over bg)"
        );
    }
}
