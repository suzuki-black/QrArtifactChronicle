//! 画像合成（docs/05）。tiny-skia CPU。6層を固定順で合成（docs/05 5.2 / docs/00 0.4.3）。
//! Step 1 ではベース画像をアセットから読まず【手続き的に描画】する（実アセットは Step 3）。
//! 合成順序: baseImage → HSV → dirt → damage → preservation → background。
//!
//! 注: ピクセルの厳密一致は決定論の必須要件ではない（同一性は artifactHash／属性, docs/08 8.6）。
//! ただし同一環境では決定論的（配置は artifact_hash 由来の固定PRNG）。

use crate::db::BaseArtifact;
use qrac_core::types::DerivedAttributes;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub const CANVAS_W: u32 = 512;
pub const CANVAS_H: u32 = 512;

/// 配置ジッタ用の小さな決定論PRNG（xorshift64）。artifact_hash から種を作る。
struct Rng(u64);
impl Rng {
    fn from_hash(hex: &str) -> Self {
        let mut s: u64 = 0xcbf29ce484222325;
        for b in hex.bytes() {
            s ^= b as u64;
            s = s.wrapping_mul(0x100000001b3);
        }
        Rng(if s == 0 { 0x9e3779b97f4a7c15 } else { s })
    }
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    fn frac(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + self.frac() * (b - a)
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// 文明ごとのベース色相。
fn civ_base_hue(civ: &str) -> f32 {
    match civ {
        "desert" => 40.0,
        "ocean" => 200.0,
        "mountain" => 120.0,
        "machine" => 210.0,
        "organic" => 95.0,
        _ => 30.0,
    }
}

fn solid(r: u8, g: u8, b: u8, a: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(r, g, b, a);
    p.anti_alias = true;
    p
}

/// カテゴリごとの被写体シルエット。
fn shape_path(category: &str) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    let (cx, cy) = (256.0_f32, 256.0_f32);
    match category {
        "weapon" => {
            // 縦長のひし形（刃）
            pb.move_to(cx, 70.0);
            pb.line_to(cx + 46.0, 300.0);
            pb.line_to(cx, 450.0);
            pb.line_to(cx - 46.0, 300.0);
            pb.close();
        }
        "ritual" => {
            pb.push_circle(cx, cy, 150.0);
        }
        "daily" => {
            // 壺（角丸長方形を矩形で近似）
            if let Some(r) = tiny_skia::Rect::from_xywh(cx - 100.0, cy - 130.0, 200.0, 260.0) {
                pb.push_rect(r);
            }
        }
        "architecture" => {
            // 台形（建築断片）
            pb.move_to(cx - 110.0, 380.0);
            pb.line_to(cx + 110.0, 380.0);
            pb.line_to(cx + 70.0, 150.0);
            pb.line_to(cx - 70.0, 150.0);
            pb.close();
        }
        "inscription" => {
            if let Some(r) = tiny_skia::Rect::from_xywh(cx - 100.0, cy - 150.0, 200.0, 300.0) {
                pb.push_rect(r);
            }
        }
        _ => {
            // machine_part: 六角形
            for i in 0..6 {
                let a = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::FRAC_PI_2;
                let (x, y) = (cx + 150.0 * a.cos(), cy + 150.0 * a.sin());
                if i == 0 {
                    pb.move_to(x, y);
                } else {
                    pb.line_to(x, y);
                }
            }
            pb.close();
        }
    }
    pb.finish().unwrap()
}

/// ベーススプライト（被写体のみ・背景透過・civの自然なベース色）。docs/05 層1。
/// アセット生成ツール(qrac-assetgen)が出力する「ベース画像」相当。
/// 実行時 compose はこれ（またはアーティスト製アセット）を読み込み、HSV(層2)で色違いを作る。
pub fn render_base_sprite(civ: &str, category: &str) -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap(); // 透過
    let id = Transform::identity();
    // civ の自然なベース色（中彩度）。色違いは実行時 HSV 補正で生む（docs/04 4.1）。
    let (r, g, b) = hsv_to_rgb(civ_base_hue(civ), 0.55, 0.70);
    let shape = shape_path(category);
    pm.fill_path(&shape, &solid(r, g, b, 255), FillRule::Winding, id, None);
    let edge = solid(
        (r as f32 * 0.6) as u8,
        (g as f32 * 0.6) as u8,
        (b as f32 * 0.6) as u8,
        255,
    );
    pm.stroke_path(
        &shape,
        &edge,
        &Stroke {
            width: 4.0,
            ..Default::default()
        },
        id,
        None,
    );
    pm
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { d / max };
    (h, s, max)
}

/// 層2: baseImage にHSV補正を適用（docs/05 5.2 / docs/04 4.1）。透明画素は不変。
fn apply_hsv(pm: &mut Pixmap, cm: &qrac_core::types::ColorMod) {
    for px in pm.pixels_mut() {
        let c = px.demultiply();
        if c.alpha() == 0 {
            continue;
        }
        let (h, s, v) = rgb_to_hsv(c.red(), c.green(), c.blue());
        let nh = h + cm.h_shift as f32;
        let ns = (s * cm.s_mul as f32).clamp(0.0, 1.0);
        let nv = (v * cm.v_mul as f32).clamp(0.0, 1.0);
        let (r, g, b) = hsv_to_rgb(nh, ns, nv);
        *px = tiny_skia::ColorU8::from_rgba(r, g, b, c.alpha()).premultiply();
    }
}

/// 6層を合成して Pixmap を返す。
/// `sprite_png`: ベース画像(層1)のPNGバイト列。None なら手続き生成にフォールバック。
pub fn render(attr: &DerivedAttributes, base: &BaseArtifact, sprite_png: Option<&[u8]>) -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let mut rng = Rng::from_hash(&attr.artifact_hash);
    let id = Transform::identity();
    let shape = shape_path(&base.category);

    // 層6(背景)を最初に塗ってから被写体を重ねる（最背面＝先に描く）。museum スタイル固定。
    pm.fill(Color::from_rgba8(238, 236, 230, 255));

    // 層1: baseImage を取得（アセット読込 or 手続き生成）。層2: HSV補正を適用。
    let mut subject = match sprite_png.and_then(|b| Pixmap::decode_png(b).ok()) {
        Some(sp) => sp,
        None => render_base_sprite(&base.civ, &base.category),
    };
    apply_hsv(&mut subject, &attr.color_mod);
    pm.draw_pixmap(
        0,
        0,
        subject.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        id,
        None,
    );

    // 層3: dirt（汚れ）。種類で色、preservation で強度。
    if base_dirt_visible(&attr.dirt_layer_id) {
        let (dr, dg, db_, _) = dirt_color(&attr.dirt_layer_id);
        let strength = dirt_strength(attr.preservation_score);
        let alpha = (strength * 200.0) as u8;
        let blobs = 6 + (strength * 6.0) as u32;
        for _ in 0..blobs {
            let x = rng.range(150.0, 362.0);
            let y = rng.range(120.0, 400.0);
            let rad = rng.range(18.0, 46.0);
            let mut pb = PathBuilder::new();
            pb.push_circle(x, y, rad);
            if let Some(p) = pb.finish() {
                pm.fill_path(&p, &solid(dr, dg, db_, alpha), FillRule::Winding, id, None);
            }
        }
    }

    // 層4: damage（破損）。
    let dmg_alpha = (damage_strength(attr.preservation_score) * 220.0) as u8;
    if attr.damage.wear {
        // 摩耗: 全体を薄い明色で曇らせる
        pm.fill_path(
            &shape,
            &solid(255, 255, 255, (dmg_alpha / 3).max(20)),
            FillRule::Winding,
            id,
            None,
        );
    }
    if attr.damage.crack {
        // ひび: ジグザグの暗線
        let mut pb = PathBuilder::new();
        let mut y = 120.0;
        pb.move_to(256.0, y);
        while y < 420.0 {
            y += rng.range(30.0, 60.0);
            pb.line_to(rng.range(200.0, 312.0), y);
        }
        if let Some(p) = pb.finish() {
            pm.stroke_path(
                &p,
                &solid(25, 20, 18, dmg_alpha),
                &Stroke {
                    width: 3.0,
                    ..Default::default()
                },
                id,
                None,
            );
        }
    }
    if attr.damage.chip {
        // 欠け: 角を背景色の三角で削る
        let mut pb = PathBuilder::new();
        pb.move_to(330.0, 110.0);
        pb.line_to(420.0, 110.0);
        pb.line_to(420.0, 200.0);
        pb.close();
        if let Some(p) = pb.finish() {
            pm.fill_path(&p, &solid(238, 236, 230, 255), FillRule::Winding, id, None);
        }
    }

    // 層5: preservation（保存状態）。低いほど全体をくすませる。
    // ※ Pixmap::fill は上書きなので使わない。全面矩形を fill_path でブレンド。
    let dust = ((1.0 - attr.preservation_score) * 70.0) as u8;
    if dust > 0 {
        if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, CANVAS_W as f32, CANVAS_H as f32) {
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(p) = pb.finish() {
                pm.fill_path(&p, &solid(120, 110, 95, dust), FillRule::Winding, id, None);
            }
        }
    }

    pm
}

fn base_dirt_visible(dirt: &str) -> bool {
    dirt != "none"
}
fn dirt_color(dirt: &str) -> (u8, u8, u8, u8) {
    match dirt {
        "mud" => (90, 64, 38, 255),
        "sand" => (194, 168, 110, 255),
        "soot" => (40, 36, 32, 255),
        "sea_salt" => (235, 238, 240, 255),
        "volcanic_ash" => (70, 66, 64, 255),
        _ => (120, 110, 95, 255),
    }
}
fn dirt_strength(s: f64) -> f32 {
    (0.9 - 0.7 * s as f32).clamp(0.1, 0.9)
}
fn damage_strength(s: f64) -> f32 {
    (0.9 - 0.6 * s as f32).clamp(0.15, 0.9)
}

/// 直 RGBA8（非プリマルチプライ）バイト列。
pub struct RgbaBuf {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn render_rgba(
    attr: &DerivedAttributes,
    base: &BaseArtifact,
    sprite_png: Option<&[u8]>,
) -> RgbaBuf {
    let pm = render(attr, base, sprite_png);
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
    sprite_png: Option<&[u8]>,
) -> Vec<u8> {
    render(attr, base, sprite_png)
        .encode_png()
        .expect("encode_png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ArtifactDb;
    use qrac_core::derive_from_string;
    use qrac_core::hash::make_seed;

    #[test]
    fn renders_nonblank_and_deterministic() {
        let db = ArtifactDb::open_in_memory().unwrap();
        db.seed_full().unwrap();
        let attr = derive_from_string("https://example.com/welcome", None);
        let seed = make_seed(&qrac_core::normalize::normalize_key(
            b"https://example.com/welcome",
        ));
        let base = db.select_base(
            &seed,
            &attr.civ,
            &attr.era,
            &attr.category,
            attr.base_rarity,
        );

        let png1 = render_png(&attr, &base, None);
        let png2 = render_png(&attr, &base, None);
        assert!(png1.len() > 1000, "png too small: {}", png1.len());
        assert_eq!(png1, png2, "render must be deterministic");

        // アセット読込経路（手続きスプライトを bytes 化して渡す）でも描画される
        let sprite = render_base_sprite(&base.civ, &base.category)
            .encode_png()
            .unwrap();
        let from_asset = render_png(&attr, &base, Some(&sprite));
        assert!(from_asset.len() > 1000);

        // 中心ピクセル(256,256)は被写体内なので背景色であってはならない
        // （preservation層が被写体を上書きしていた退行を検出）。
        let buf = render_rgba(&attr, &base, None);
        let ci = ((256 * CANVAS_W + 256) * 4) as usize;
        let center_is_bg =
            buf.rgba[ci] == 238 && buf.rgba[ci + 1] == 236 && buf.rgba[ci + 2] == 230;
        assert!(
            !center_is_bg,
            "subject not drawn at center (got {:?})",
            &buf.rgba[ci..ci + 4]
        );
    }

    #[test]
    fn sprite_bytes_actually_drive_layer1() {
        let db = ArtifactDb::open_in_memory().unwrap();
        db.seed_full().unwrap();
        let attr = derive_from_string("asset-probe", None);
        let seed = make_seed(&qrac_core::normalize::normalize_key(b"asset-probe"));
        let base = db.select_base(
            &seed,
            &attr.civ,
            &attr.era,
            &attr.category,
            attr.base_rarity,
        );

        // 全面不透明のスプライトを渡すと、手続き(None)では背景だった隅も被写体で塗られる。
        let mut full = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
        full.fill(Color::from_rgba8(200, 30, 30, 255));
        let full_png = full.encode_png().unwrap();

        let with_asset = render_rgba(&attr, &base, Some(&full_png));
        let procedural = render_rgba(&attr, &base, None);
        let corner = ((10 * CANVAS_W + 10) * 4) as usize;
        assert_ne!(
            &with_asset.rgba[corner..corner + 4],
            &procedural.rgba[corner..corner + 4],
            "sprite_png must drive layer 1 (corner pixel should differ)"
        );
    }
}
