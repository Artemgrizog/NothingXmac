//! Vector renders of every device family (so the app works offline).
//! A PNG dropped into `~/Library/Application Support/NothingX/images/<code>.png`
//! (e.g. `B170.png`) replaces the built-in drawing.

use crate::catalog::{Kind, Model};
use tiny_skia::*;

type Rgb = [u8; 3];

fn col(c: Rgb, a: u8) -> Color {
    Color::from_rgba8(c[0], c[1], c[2], a)
}

fn solid(c: Rgb, a: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(col(c, a));
    p.anti_alias = true;
    p
}

fn vgrad(y0: f32, y1: f32, top: Rgb, bottom: Rgb, a: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.anti_alias = true;
    p.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y1),
        vec![GradientStop::new(0.0, col(top, a)), GradientStop::new(1.0, col(bottom, a))],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or(Shader::SolidColor(col(top, a)));
    p
}

fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = r * 0.5523;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish().unwrap()
}

fn circle(cx: f32, cy: f32, r: f32) -> Path {
    PathBuilder::from_circle(cx, cy, r).unwrap()
}

fn stroke(w: f32) -> Stroke {
    Stroke { width: w, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Default::default() }
}

struct Pal {
    hi: Rgb,
    lo: Rgb,
    pad: Rgb,
    edge: Rgb,
}

fn pal(dark: bool) -> Pal {
    if dark {
        Pal { hi: [78, 78, 84], lo: [38, 38, 42], pad: [20, 20, 22], edge: [110, 110, 118] }
    } else {
        Pal { hi: [240, 240, 243], lo: [176, 176, 184], pad: [28, 28, 30], edge: [255, 255, 255] }
    }
}

const RED: Rgb = [236, 33, 39];

fn over_ear(pm: &mut Canvas, dark: bool, accent: Option<Rgb>) {
    let p = pal(dark);
    let id = Transform::identity();
    // headband
    let mut pb = PathBuilder::new();
    pb.move_to(112.0, 270.0);
    pb.cubic_to(112.0, 36.0, 400.0, 36.0, 400.0, 270.0);
    let band = pb.finish().unwrap();
    pm.stroke_path(&band, &solid(p.lo, 255), &stroke(34.0), id, None);
    pm.stroke_path(&band, &vgrad(40.0, 200.0, p.hi, p.lo, 255), &stroke(26.0), id, None);
    let mut pb = PathBuilder::new();
    pb.move_to(170.0, 116.0);
    pb.quad_to(256.0, 84.0, 342.0, 116.0);
    pm.stroke_path(&pb.finish().unwrap(), &solid(p.pad, 255), &stroke(24.0), id, None);

    for side in 0..2 {
        let mirror = side == 1;
        let mx = |x: f32, w: f32| if mirror { 512.0 - x - w } else { x };
        let cx = if mirror { 395.0 } else { 117.0 };
        let tf = Transform::from_rotate_at(if mirror { 8.0 } else { -8.0 }, cx, 360.0);
        // pad (inner cushion)
        let pad = rrect(mx(104.0, 104.0), 258.0, 104.0, 196.0, 52.0);
        pm.fill_path(&pad, &vgrad(258.0, 454.0, [48, 48, 52], p.pad, 255), FillRule::Winding, tf, None);
        // shell
        let shell = rrect(mx(58.0, 76.0), 266.0, 76.0, 190.0, 34.0);
        pm.fill_path(&shell, &vgrad(266.0, 456.0, p.hi, p.lo, 255), FillRule::Winding, tf, None);
        pm.stroke_path(&shell, &solid(p.edge, 90), &stroke(2.0), tf, None);
        // clear window with detail
        let win = rrect(mx(68.0, 56.0), 300.0, 56.0, 120.0, 20.0);
        pm.fill_path(&win, &solid(if dark { [0, 0, 0] } else { [255, 255, 255] }, 70), FillRule::Winding, tf, None);
        for i in 0..5 {
            let y = 330.0 + i as f32 * 12.0;
            let l = rrect(mx(78.0, 36.0), y, 36.0, 3.0, 1.5);
            pm.fill_path(&l, &solid(if dark { [150, 150, 158] } else { [120, 120, 128] }, 120), FillRule::Winding, tf, None);
        }
        if let Some(a) = accent {
            let band = rrect(mx(58.0, 76.0), 418.0, 76.0, 38.0, 20.0);
            pm.fill_path(&band, &solid(a, 255), FillRule::Winding, tf, None);
        }
        if mirror {
            pm.fill_path(&circle(mx(86.0, 12.0) + 6.0, 286.0, 7.0), &solid(RED, 255), FillRule::Winding, tf, None);
        }
    }
}

fn stem_bud(pm: &mut Canvas, cx: f32, cy: f32, s: f32, flip: bool, angle: f32, dark: bool, accent: Option<Rgb>) {
    let p = pal(dark);
    let sx = if flip { -s } else { s };
    let tf = Transform::from_translate(cx, cy).pre_concat(Transform::from_rotate(angle)).pre_concat(Transform::from_scale(sx, s));
    // stem (translucent shell + dark core)
    let stem = rrect(-30.0, -40.0, 60.0, 250.0, 30.0);
    pm.fill_path(&stem, &vgrad(-40.0, 210.0, p.hi, p.lo, 235), FillRule::Winding, tf, None);
    pm.stroke_path(&stem, &solid(p.edge, 110), &stroke(2.5), tf, None);
    let core = rrect(-17.0, -8.0, 34.0, 190.0, 17.0);
    pm.fill_path(&core, &solid([14, 14, 16], 255), FillRule::Winding, tf, None);
    for i in 0..4 {
        let l = rrect(-9.0, 40.0 + i as f32 * 22.0, 18.0, 3.0, 1.5);
        pm.fill_path(&l, &solid([90, 90, 96], 255), FillRule::Winding, tf, None);
    }
    pm.fill_path(&circle(0.0, 8.0, 9.0), &solid(RED, 255), FillRule::Winding, tf, None);
    // head + tip
    let head = rrect(-52.0, -128.0, 108.0, 112.0, 50.0);
    pm.fill_path(&head, &vgrad(-128.0, -16.0, p.hi, p.lo, 255), FillRule::Winding, tf, None);
    pm.stroke_path(&head, &solid(p.edge, 110), &stroke(2.5), tf, None);
    let tip = accent.unwrap_or(if dark { [60, 60, 64] } else { [244, 244, 246] });
    pm.fill_path(&circle(-20.0, -86.0, 56.0), &solid(tip, 255), FillRule::Winding, tf, None);
    pm.fill_path(&circle(-26.0, -86.0, 28.0), &solid([40, 40, 44], 255), FillRule::Winding, tf, None);
    pm.fill_path(&circle(-26.0, -86.0, 12.0), &solid([18, 18, 20], 255), FillRule::Winding, tf, None);
}

fn stems(pm: &mut Canvas, dark: bool, accent: Option<Rgb>) {
    stem_bud(pm, 176.0, 168.0, 0.95, false, -16.0, dark, accent);
    stem_bud(pm, 346.0, 168.0, 0.95, true, 16.0, dark, accent);
}

fn buds(pm: &mut Canvas, dark: bool, accent: Option<Rgb>) {
    let body: Rgb = if dark { [34, 34, 38] } else { [236, 236, 240] };
    let a = accent.unwrap_or([255, 98, 40]);
    for side in 0..2 {
        let mirror = side == 1;
        let cx = if mirror { 346.0 } else { 166.0 };
        let tf = Transform::from_translate(cx, 250.0).pre_concat(Transform::from_rotate(if mirror { 14.0 } else { -14.0 }));
        let stem = rrect(-24.0, -10.0, 48.0, 180.0, 24.0);
        pm.fill_path(&stem, &vgrad(-10.0, 170.0, body, if dark { [14, 14, 16] } else { [190, 190, 198] }, 255), FillRule::Winding, tf, None);
        pm.fill_path(&circle(0.0, -40.0, 78.0), &vgrad(-118.0, 40.0, body, if dark { [20, 20, 24] } else { [200, 200, 208] }, 255), FillRule::Winding, tf, None);
        pm.fill_path(&circle(0.0, -40.0, 56.0), &solid(a, 255), FillRule::Winding, tf, None);
        pm.fill_path(&circle(0.0, -40.0, 30.0), &solid(if dark { [10, 10, 12] } else { [250, 250, 252] }, 255), FillRule::Winding, tf, None);
    }
}

fn neckband(pm: &mut Canvas, dark: bool, accent: Option<Rgb>) {
    let id = Transform::identity();
    let band: Rgb = if dark { [44, 44, 48] } else { [226, 226, 230] };
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 110.0);
    pb.cubic_to(86.0, 470.0, 426.0, 470.0, 412.0, 110.0);
    pm.stroke_path(&pb.finish().unwrap(), &solid(band, 255), &stroke(30.0), id, None);
    let a = accent.unwrap_or([255, 98, 40]);
    pm.fill_path(&rrect(96.0, 380.0, 56.0, 40.0, 16.0), &solid(a, 255), FillRule::Winding, Transform::from_rotate_at(30.0, 124.0, 400.0), None);
    pm.fill_path(&rrect(360.0, 380.0, 56.0, 40.0, 16.0), &solid(a, 255), FillRule::Winding, Transform::from_rotate_at(-30.0, 388.0, 400.0), None);
    for x in [100.0f32, 412.0] {
        pm.fill_path(&circle(x, 96.0, 36.0), &solid(band, 255), FillRule::Winding, id, None);
        pm.fill_path(&circle(x, 96.0, 16.0), &solid([18, 18, 20], 255), FillRule::Winding, id, None);
    }
}

fn open_ear(pm: &mut Canvas, dark: bool) {
    let id = Transform::identity();
    let body: Rgb = if dark { [60, 60, 64] } else { [238, 238, 242] };
    let mut pb = PathBuilder::new();
    pb.move_to(330.0, 160.0);
    pb.cubic_to(230.0, 40.0, 110.0, 150.0, 150.0, 280.0);
    pb.cubic_to(176.0, 360.0, 262.0, 380.0, 330.0, 340.0);
    pm.stroke_path(&pb.finish().unwrap(), &vgrad(60.0, 380.0, body, [176, 176, 184], 255), &stroke(28.0), id, None);
    pm.fill_path(&rrect(290.0, 318.0, 150.0, 48.0, 24.0), &vgrad(318.0, 366.0, [236, 236, 240], [150, 150, 158], 255), FillRule::Winding, id, None);
    pm.fill_path(&circle(330.0, 170.0, 62.0), &solid([230, 230, 234], 255), FillRule::Winding, id, None);
    pm.fill_path(&circle(330.0, 170.0, 44.0), &solid([16, 16, 18], 255), FillRule::Winding, id, None);
    pm.fill_path(&circle(330.0, 170.0, 10.0), &solid(RED, 255), FillRule::Winding, id, None);
}

pub fn draw(kind: Kind, dark: bool, accent: Option<[u8; 3]>, size: u32) -> Pixmap {
    // Artwork is designed on a 512x512 canvas; `Canvas` scales every call to `size`.
    let mut c = Canvas { pm: Pixmap::new(size, size).unwrap(), base: Transform::from_scale(size as f32 / 512.0, size as f32 / 512.0) };
    match kind {
        Kind::Over => over_ear(&mut c, dark, accent),
        Kind::Stem => stems(&mut c, dark, accent),
        Kind::Bud => buds(&mut c, dark, accent),
        Kind::Neck => neckband(&mut c, dark, accent),
        Kind::Open => open_ear(&mut c, dark),
    }
    c.pm
}

/// Pixmap + base transform so the 512-unit artwork renders crisply at any size.
struct Canvas {
    pm: Pixmap,
    base: Transform,
}

impl Canvas {
    fn fill_path(&mut self, path: &Path, paint: &Paint, rule: FillRule, tf: Transform, _m: Option<&Mask>) {
        self.pm.fill_path(path, paint, rule, self.base.pre_concat(tf), None);
    }
    fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke, tf: Transform, _m: Option<&Mask>) {
        self.pm.stroke_path(path, paint, stroke, self.base.pre_concat(tf), None);
    }
}

fn override_path(code: &str) -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let p = std::path::PathBuf::from(home).join("Library/Application Support/NothingX/images").join(format!("{code}.png"));
    p.exists().then_some(p)
}

/// Official product renders shipped with the app (see assets/devices/README.md).
fn bundled(code: &str) -> Option<&'static [u8]> {
    Some(match code {
        "B155" => include_bytes!("../assets/devices/B155.png"),
        "B157" => include_bytes!("../assets/devices/B157.png"),
        "B163" => include_bytes!("../assets/devices/B163.png"),
        "B164" => include_bytes!("../assets/devices/B164.png"),
        "B168" => include_bytes!("../assets/devices/B168.png"),
        "B170" => include_bytes!("../assets/devices/B170.png"),
        "B171" => include_bytes!("../assets/devices/B171.png"),
        "B172" => include_bytes!("../assets/devices/B172.png"),
        "B173" => include_bytes!("../assets/devices/B173.png"),
        "B174" => include_bytes!("../assets/devices/B174.png"),
        "B175" => include_bytes!("../assets/devices/B175.png"),
        "B179" => include_bytes!("../assets/devices/B179.png"),
        "B17P" => include_bytes!("../assets/devices/B17P.png"),
        "B181" => include_bytes!("../assets/devices/B181.png"),
        "B183" => include_bytes!("../assets/devices/B183.png"),
        "B184" => include_bytes!("../assets/devices/B184.png"),
        "B185" => include_bytes!("../assets/devices/B185.png"),
        "B186" => include_bytes!("../assets/devices/B186.png"),
        "B190" => include_bytes!("../assets/devices/B190.png"),
        "B193" => include_bytes!("../assets/devices/B193.png"),
        _ => return None,
    })
}

/// Scale `src` to fit a `size` x `size` square, centred.
fn fit(src: &Pixmap, size: u32) -> Pixmap {
    let mut out = Pixmap::new(size, size).unwrap();
    let s = (size as f32 / src.width() as f32).min(size as f32 / src.height() as f32);
    let dx = (size as f32 - src.width() as f32 * s) / 2.0;
    let dy = (size as f32 - src.height() as f32 * s) / 2.0;
    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;
    out.draw_pixmap(0, 0, src.as_ref(), &paint, Transform::from_translate(dx, dy).pre_scale(s, s), None);
    out
}

/// Image for a model: user override, else the bundled render, else the built-in drawing.
pub fn model_pixmap(m: &Model, size: u32) -> Pixmap {
    let user = override_path(m.code).and_then(|p| std::fs::read(p).ok());
    let bytes = user.as_deref().or_else(|| bundled(m.code));
    if let Some(pm) = bytes.and_then(|b| Pixmap::decode_png(b).ok()) {
        return fit(&pm, size);
    }
    draw(m.kind, m.dark, m.accent, size)
}

pub fn model_image(m: &Model, size: u32) -> slint::Image {
    crate::dotmatrix::to_image(&model_pixmap(m, size))
}

/// 1024x1024 app icon: black squircle, dot-matrix "NX" and a red dot.
pub fn app_icon() -> Pixmap {
    let mut pm = Pixmap::new(1024, 1024).unwrap();
    let id = Transform::identity();
    pm.fill_path(&rrect(100.0, 100.0, 824.0, 824.0, 190.0), &vgrad(100.0, 924.0, [30, 30, 32], [4, 4, 5], 255), FillRule::Winding, id, None);
    let text = crate::dotmatrix::render_pixmap("NX", 50.0, 18.0, [255, 255, 255], true);
    let x = (1024 - text.width() as i32) / 2;
    let y = (1024 - text.height() as i32) / 2 + 20;
    pm.draw_pixmap(x, y, text.as_ref(), &PixmapPaint::default(), id, None);
    pm.fill_path(&circle(770.0, 250.0, 34.0), &solid(RED, 255), FillRule::Winding, id, None);
    pm
}

/// DMG window background (`scale` 1 or 2): black, dot-matrix arrow from the app to Applications.
pub fn dmg_background(scale: f32) -> Pixmap {
    let (w, h) = (640.0 * scale, 400.0 * scale);
    let mut pm = Pixmap::new(w as u32, h as u32).unwrap();
    pm.fill(Color::from_rgba8(0, 0, 0, 255));
    let base = Transform::from_scale(scale, scale);
    // faint dot grid
    let mut grid = PathBuilder::new();
    let mut y = 12.0;
    while y < 400.0 {
        let mut x = 12.0;
        while x < 640.0 {
            grid.push_circle(x, y, 0.9);
            x += 16.0;
        }
        y += 16.0;
    }
    if let Some(p) = grid.finish() {
        pm.fill_path(&p, &solid([255, 255, 255], 18), FillRule::Winding, base, None);
    }
    // dotted arrow between the icon slots (icons sit at x=170 and x=470, y=190)
    let mut arrow = PathBuilder::new();
    let mut x = 262.0;
    while x <= 372.0 {
        arrow.push_circle(x, 190.0, 3.2);
        x += 11.0;
    }
    for i in 1..=4 {
        let d = i as f32 * 9.0;
        arrow.push_circle(372.0 - d, 190.0 - d, 3.2);
        arrow.push_circle(372.0 - d, 190.0 + d, 3.2);
    }
    if let Some(p) = arrow.finish() {
        pm.fill_path(&p, &solid([255, 255, 255], 235), FillRule::Winding, base, None);
    }
    pm.fill_path(&circle(372.0, 190.0, 4.2), &solid(RED, 255), FillRule::Winding, base, None);
    // captions in dot-matrix; the last two lines explain the Gatekeeper prompt of an
    // app that is not notarized
    for (text, y, dot, alpha) in [
        ("NOTHINGX FOR MAC", 38.0, 3.2, 255u8),
        ("UNOFFICIAL", 86.0, 1.6, 110),
        ("DRAG TO APPLICATIONS", 292.0, 2.2, 210),
        ("BLOCKED ON FIRST LAUNCH?", 334.0, 1.4, 160),
        ("SETTINGS > PRIVACY & SECURITY > OPEN ANYWAY", 356.0, 1.4, 160),
    ] {
        let t = crate::dotmatrix::render_pixmap(text, dot * scale, dot * 0.55 * scale, [255, 255, 255], false);
        let mut paint = PixmapPaint::default();
        paint.opacity = alpha as f32 / 255.0;
        pm.draw_pixmap(((w - t.width() as f32) / 2.0) as i32, (y * scale) as i32, t.as_ref(), &paint, Transform::identity(), None);
    }
    pm
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_model_has_a_bundled_picture() {
        for m in crate::catalog::MODELS {
            let png = super::bundled(m.code).unwrap_or_else(|| panic!("no image for {}", m.code));
            let pm = tiny_skia::Pixmap::decode_png(png).expect("valid png");
            assert!(pm.width() >= 400 && pm.height() >= 400, "{} is {}x{}", m.code, pm.width(), pm.height());
        }
    }
}
