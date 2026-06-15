//! 手続きアート生成（docs/05）。陰影付きベース＋汚れ/破損/背景の各レイヤーを生成する。
//! qrac-assetgen がこれらを PNG に書き出し、compose は同じ関数を実行時フォールバックにも使う。
//! ※ 写真ではなく「質感のあるスタイライズ」。決定論の必須要件ではない（同一性は属性/hash）。

use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Mask, MaskType, Paint, PathBuilder, Pixmap,
    Point, SpreadMode, Stroke, Transform,
};

pub const CANVAS_W: u32 = 512;
pub const CANVAS_H: u32 = 512;
const CX: f32 = 256.0;

/// 決定論PRNG（xorshift64）。
pub struct Rng(u64);
impl Rng {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        Rng(if h == 0 { 0x9e3779b97f4a7c15 } else { h })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    pub fn frac(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + self.frac() * (b - a)
    }
}

fn solid(r: u8, g: u8, b: u8, a: u8) -> Paint<'static> {
    let mut p = Paint {
        anti_alias: true,
        ..Default::default()
    };
    p.set_color_rgba8(r, g, b, a);
    p
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
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
    let f = |t: f32| ((t + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (f(r), f(g), f(b))
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
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
    (
        (if h < 0.0 { h + 360.0 } else { h }),
        if max == 0.0 { 0.0 } else { d / max },
        max,
    )
}

pub fn civ_base_hue(civ: &str) -> f32 {
    match civ {
        "desert" => 38.0,
        "ocean" => 200.0,
        "mountain" => 120.0,
        "machine" => 210.0,
        "organic" => 95.0,
        _ => 30.0,
    }
}

/// カテゴリ別の被写体シルエット。
pub fn shape_path(category: &str) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    let (cx, cy) = (CX, 256.0);
    match category {
        "weapon" => {
            pb.move_to(cx, 64.0);
            pb.line_to(cx + 46.0, 300.0);
            pb.line_to(cx, 452.0);
            pb.line_to(cx - 46.0, 300.0);
            pb.close();
        }
        "ritual" => {
            pb.push_circle(cx, cy, 150.0);
        }
        "daily" => {
            if let Some(r) = tiny_skia::Rect::from_xywh(cx - 100.0, cy - 130.0, 200.0, 260.0) {
                pb.push_rect(r);
            }
        }
        "architecture" => {
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

fn shape_mask(shape: &tiny_skia::Path) -> Mask {
    let mut sp = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    sp.fill_path(
        shape,
        &solid(255, 255, 255, 255),
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    Mask::from_pixmap(sp.as_ref(), MaskType::Alpha)
}

/// 層1: 陰影＋質感つきベース被写体（背景透過、civの自然色）。
pub fn base_sprite(civ: &str, category: &str) -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let shape = shape_path(category);
    let mask = shape_mask(&shape);

    // 上→下の縦グラデで立体感（上が明るい）
    let hue = civ_base_hue(civ);
    let (tr, tg, tb) = hsv_to_rgb(hue, 0.42, 0.88);
    let (br, bg, bb) = hsv_to_rgb(hue, 0.58, 0.44);
    let grad = LinearGradient::new(
        Point::from_xy(CX, 70.0),
        Point::from_xy(CX, 452.0),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(tr, tg, tb, 255)),
            GradientStop::new(1.0, Color::from_rgba8(br, bg, bb, 255)),
        ],
        SpreadMode::Pad,
        id,
    );
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    if let Some(g) = grad {
        paint.shader = g;
    } else {
        paint.set_color_rgba8(tr, tg, tb, 255);
    }
    pm.fill_path(&shape, &paint, FillRule::Winding, id, None);

    // 斑（質感）: シルエット内に微小ドットを散らす
    let mut rng = Rng::from_str(&format!("{civ}/{category}"));
    for _ in 0..170 {
        let x = rng.range(70.0, 442.0);
        let y = rng.range(70.0, 442.0);
        let rad = rng.range(1.0, 3.2);
        let a = rng.range(8.0, 34.0) as u8;
        let c = if rng.frac() < 0.5 {
            solid(0, 0, 0, a)
        } else {
            solid(255, 255, 255, a)
        };
        let mut pb = PathBuilder::new();
        pb.push_circle(x, y, rad);
        if let Some(p) = pb.finish() {
            pm.fill_path(&p, &c, FillRule::Winding, id, Some(&mask));
        }
    }

    // 縁取り（暗）
    let edge = solid(
        (br as f32 * 0.55) as u8,
        (bg as f32 * 0.55) as u8,
        (bb as f32 * 0.55) as u8,
        255,
    );
    pm.stroke_path(
        &shape,
        &edge,
        &Stroke {
            width: 5.0,
            ..Default::default()
        },
        id,
        None,
    );
    pm
}

fn dirt_rgb(dirt: &str) -> (u8, u8, u8) {
    match dirt {
        "mud" => (84, 60, 36),
        "sand" => (196, 170, 112),
        "soot" => (34, 30, 28),
        "sea_salt" => (236, 238, 240),
        "volcanic_ash" => (66, 62, 60),
        _ => (110, 100, 88),
    }
}

/// 層3: 汚れテクスチャ（全面透過。compose 側で被写体にマスクして合成）。none は None。
pub fn dirt_texture(dirt: &str) -> Option<Pixmap> {
    if dirt == "none" {
        return None;
    }
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let (r, g, b) = dirt_rgb(dirt);
    let mut rng = Rng::from_str(&format!("dirt/{dirt}"));
    let blobs = if dirt == "sea_salt" { 90 } else { 46 };
    for _ in 0..blobs {
        let x = rng.range(0.0, 512.0);
        let y = rng.range(0.0, 512.0);
        let rad = if dirt == "sea_salt" {
            rng.range(1.0, 4.0)
        } else {
            rng.range(14.0, 64.0)
        };
        let a = rng.range(40.0, 150.0) as u8;
        let mut pb = PathBuilder::new();
        pb.push_circle(x, y, rad);
        if let Some(p) = pb.finish() {
            pm.fill_path(&p, &solid(r, g, b, a), FillRule::Winding, id, None);
        }
    }
    Some(pm)
}

/// 層4(摩耗): 明るいスクラッチ（SoftLight 合成想定）。
pub fn wear_layer() -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let mut rng = Rng::from_str("damage/wear");
    for _ in 0..26 {
        let x0 = rng.range(60.0, 452.0);
        let y0 = rng.range(60.0, 452.0);
        let len = rng.range(20.0, 90.0);
        let ang = rng.range(-0.6, 0.6);
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x0 + len * ang.cos(), y0 + len * ang.sin());
        if let Some(p) = pb.finish() {
            let a = rng.range(60.0, 150.0) as u8;
            pm.stroke_path(
                &p,
                &solid(255, 250, 240, a),
                &Stroke {
                    width: rng.range(1.0, 2.5),
                    ..Default::default()
                },
                id,
                None,
            );
        }
    }
    pm
}

/// 層4(ひび): 暗い亀裂網（Multiply 合成想定）。
pub fn crack_layer() -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let mut rng = Rng::from_str("damage/crack");
    let paint = solid(20, 16, 14, 230);
    // 主幹2本＋枝
    for _ in 0..2 {
        let mut x = rng.range(180.0, 332.0);
        let mut y = 110.0;
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        while y < 410.0 {
            y += rng.range(28.0, 56.0);
            x += rng.range(-40.0, 40.0);
            pb.line_to(x.clamp(120.0, 392.0), y);
        }
        if let Some(p) = pb.finish() {
            pm.stroke_path(
                &p,
                &paint,
                &Stroke {
                    width: 3.0,
                    ..Default::default()
                },
                id,
                None,
            );
        }
    }
    for _ in 0..6 {
        let x0 = rng.range(160.0, 352.0);
        let y0 = rng.range(140.0, 380.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x0 + rng.range(-50.0, 50.0), y0 + rng.range(-40.0, 60.0));
        if let Some(p) = pb.finish() {
            pm.stroke_path(
                &p,
                &paint,
                &Stroke {
                    width: 1.6,
                    ..Default::default()
                },
                id,
                None,
            );
        }
    }
    pm
}

/// 層4(欠け): 不透明な切り欠き形状（DestinationOut で被写体を削る）。
pub fn chip_layer() -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let mut rng = Rng::from_str("damage/chip");
    let white = solid(255, 255, 255, 255);
    for _ in 0..3 {
        let cx = rng.range(110.0, 402.0);
        let cy = rng.range(110.0, 402.0);
        let mut pb = PathBuilder::new();
        let n = 5;
        for i in 0..n {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let rad = rng.range(10.0, 28.0);
            let (x, y) = (cx + rad * a.cos(), cy + rad * a.sin());
            if i == 0 {
                pb.move_to(x, y);
            } else {
                pb.line_to(x, y);
            }
        }
        pb.close();
        if let Some(p) = pb.finish() {
            pm.fill_path(&p, &white, FillRule::Winding, id, None);
        }
    }
    pm
}

/// 層6: 写真風の背景（museum / dig / catalog）。全面不透明。
pub fn background(style: &str) -> Pixmap {
    let mut pm = Pixmap::new(CANVAS_W, CANVAS_H).unwrap();
    let id = Transform::identity();
    let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, 512.0, 512.0).unwrap();
    let (top, bot, speckle) = match style {
        "dig" => ((150, 120, 84), (96, 74, 48), true), // 発掘現場（土）
        "catalog" => ((250, 249, 245), (232, 230, 224), false), // 図録（清潔）
        _ => ((232, 230, 224), (198, 196, 188), false), // museum（ニュートラル）
    };
    let grad = LinearGradient::new(
        Point::from_xy(CX, 0.0),
        Point::from_xy(CX, 512.0),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(top.0, top.1, top.2, 255)),
            GradientStop::new(1.0, Color::from_rgba8(bot.0, bot.1, bot.2, 255)),
        ],
        SpreadMode::Pad,
        id,
    );
    let mut paint = Paint::default();
    if let Some(g) = grad {
        paint.shader = g;
    } else {
        paint.set_color_rgba8(top.0, top.1, top.2, 255);
    }
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    let bgpath = pb.finish().unwrap();
    pm.fill_path(&bgpath, &paint, FillRule::Winding, id, None);

    if speckle {
        let mut rng = Rng::from_str(&format!("bg/{style}"));
        for _ in 0..400 {
            let x = rng.range(0.0, 512.0);
            let y = rng.range(0.0, 512.0);
            let a = rng.range(8.0, 30.0) as u8;
            let dark = rng.frac() < 0.6;
            let c = if dark {
                solid(60, 44, 28, a)
            } else {
                solid(210, 180, 130, a)
            };
            let mut p = PathBuilder::new();
            p.push_circle(x, y, rng.range(1.0, 2.5));
            if let Some(pp) = p.finish() {
                pm.fill_path(&pp, &c, FillRule::Winding, id, None);
            }
        }
    }
    pm
}
