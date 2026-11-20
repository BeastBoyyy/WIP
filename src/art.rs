//! Each theme's own art: five pieces, one per workspace, drawn in code (original, free to ship, and re-coloured
//! by the palette you edit). They become the dashboard's art cards and the wallpapers you can pick from the bar.
//! A theme without its own pieces yet shows its procedural background on all five.
use crate::paint::{col, Cv, Rng};
use crate::theme::{Rgb, Theme};
use std::f32::consts::{PI, TAU};
use tiny_skia::{BlendMode, PathBuilder, Pixmap, Transform};

pub const PIECES: usize = 5;

/// Pictures you give a theme, kept on this PC in data\art\<id>\ and never shipped: `walls` are its desktop
/// wallpapers, `ws` one picture per workspace for the dashboard. A theme with none draws its own pieces.
pub fn own(id: &str, kind: &str) -> Vec<std::path::PathBuf> {
    let d = crate::apply::data_dir().join("art").join(id).join(kind);
    let mut v: Vec<_> = std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| ["jpg", "jpeg", "png", "webp"].contains(&e.to_ascii_lowercase().as_str())))
        .collect();
    v.sort();
    v
}
pub fn open(p: &std::path::Path) -> Option<image::RgbaImage> { image::open(p).ok().map(|i| i.to_rgba8()) }
/// a picture filling w×h, cropped to its middle, resampled properly (a 4K picture scaled straight down shimmers)
pub fn fill(img: &image::RgbaImage, w: u32, h: u32) -> Pixmap {
    let (iw, ih) = img.dimensions();
    let s = (w as f32 / iw as f32).max(h as f32 / ih as f32);
    let (cw, ch) = (((w as f32 / s).round() as u32).clamp(1, iw), ((h as f32 / s).round() as u32).clamp(1, ih));
    let crop = image::imageops::crop_imm(img, (iw - cw) / 2, (ih - ch) / 2, cw, ch).to_image();
    crate::paint::to_pixmap(&image::imageops::resize(&crop, w, h, image::imageops::FilterType::CatmullRom))
}
/// how many wallpapers a theme offers: your pictures, or its five drawn pieces
pub fn count(t: &Theme) -> usize { match own(t.id, "walls").len() { 0 => PIECES, n => n.min(30) } }

pub fn piece(t: &Theme, i: usize, w: u32, h: u32) -> Pixmap {
    if let Some(img) = own(t.id, "walls").get(i).and_then(|p| open(p)) { return fill(&img, w, h); }
    let mut c = Cv::new(w, h);
    match t.id {
        "vagabond" => ronin(&mut c, t, i),
        "fresco" => fresco(&mut c, t, i),
        "relic" => relic(&mut c, t, i),
        "inferno" => inferno(&mut c, t, i),
        "schematic" => schematic(&mut c, t, i),
        "marquee" => marquee(&mut c, t, i),
        "dossier" => dossier(&mut c, t, i),
        "xerox" => xerox(&mut c, t, i),
        "etch" => etch(&mut c, t, i),
        "dither" => dither(&mut c, t, i),
        "ghost" => ghost(&mut c, t, i),
        "y2k" => y2k(&mut c, t, i),
        _ => crate::backgrounds::draw(&mut c, t),
    }
    c.reset();
    c.px
}

/* ── shared strokes ─────────────────────────────────────────────────────────────────────────────────────── */

/// one loaded brush along a polyline: several hairs, each a little off the path, wetter at the start
fn brush(c: &mut Cv, pts: &[(f32, f32)], width: f32, colour: Rgb, a: f32, r: &mut Rng) {
    if pts.len() < 2 { return; }
    let hairs = 9;
    for k in 0..hairs {
        let off = (k as f32 / (hairs - 1) as f32 - 0.5) * width * 0.8;
        let dry = 0.35 + r.f() * 0.65;
        let mut b = PathBuilder::new();
        let n = pts.len();
        // only a run of the path for some hairs: the dry break near the end of a stroke
        let end = if r.f() < 0.4 { ((n as f32) * (0.6 + r.f() * 0.4)) as usize } else { n };
        for (j, &(x, y)) in pts[..end.max(2)].iter().enumerate() {
            let (nx, ny) = normal(pts, j);
            let jit = (r.f() - 0.5) * width * 0.08;
            let (px, py) = (x + nx * (off + jit), y + ny * (off + jit));
            if j == 0 { b.move_to(px, py) } else { b.line_to(px, py) }
        }
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(colour, a * dry)), width * (0.12 + r.f() * 0.2)); }
    }
}
fn normal(pts: &[(f32, f32)], j: usize) -> (f32, f32) {
    let (a, b) = (pts[j.saturating_sub(1)], pts[(j + 1).min(pts.len() - 1)]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = (dx * dx + dy * dy).sqrt().max(0.001);
    (-dy / l, dx / l)
}
/// a filled stroke along a path, as wide as w(t) at each point (t from 0 to 1)
fn ribbon(c: &mut Cv, pts: &[(f32, f32)], w: impl Fn(f32) -> f32, s: tiny_skia::Shader<'static>) {
    if pts.len() < 2 { return; }
    let n = pts.len();
    let side = |sign: f32, j: usize| { let (nx, ny) = normal(pts, j); let hw = w(j as f32 / (n - 1) as f32) / 2. * sign; (pts[j].0 + nx * hw, pts[j].1 + ny * hw) };
    let mut b = PathBuilder::new();
    let (x, y) = side(1., 0);
    b.move_to(x, y);
    for j in 1..n { let (x, y) = side(1., j); b.line_to(x, y); }
    for j in (0..n).rev() { let (x, y) = side(-1., j); b.line_to(x, y); }
    b.close();
    if let Some(p) = b.finish() { c.fill(&p, s); }
}
/// a leaf or a single pressed brush mark: thin, full in the middle, thin again
fn leaf(c: &mut Cv, ctrl: &[(f32, f32); 3], width: f32, s: tiny_skia::Shader<'static>) {
    let pts: Vec<(f32, f32)> = (0..=16).map(|k| {
        let t = k as f32 / 16.;
        let m = 1. - t;
        (m * m * ctrl[0].0 + 2. * m * t * ctrl[1].0 + t * t * ctrl[2].0, m * m * ctrl[0].1 + 2. * m * t * ctrl[1].1 + t * t * ctrl[2].1)
    }).collect();
    ribbon(c, &pts, |t| width * (PI * t).sin().max(0.).powf(0.7), s);
}
/// a mountain line by midpoint displacement: jagged, never the same twice
fn peaks(w: f32, base: f32, amp: f32, rough: f32, r: &mut Rng) -> Vec<(f32, f32)> {
    let n = 256;
    let mut y = vec![0f32; n + 1];
    y[0] = r.f(); y[n] = r.f();
    let (mut step, mut scale) = (n, 1f32);
    while step > 1 {
        let half = step / 2;
        let mut i = half;
        while i < n { y[i] = (y[i - half] + y[i + half]) / 2. + (r.f() - 0.5) * scale; i += step; }
        scale *= rough;
        step = half;
    }
    let (lo, hi) = y.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    (0..=n).map(|i| (w * i as f32 / n as f32, base - amp * (y[i] - lo) / (hi - lo).max(0.001))).collect()
}
/// a sumi pine: a crooked trunk and flat pads of needles reaching sideways
fn pine(c: &mut Cv, x: f32, y: f32, s: f32, tone: Rgb, r: &mut Rng) {
    let trunk: Vec<(f32, f32)> = (0..=12).map(|k| { let f = k as f32 / 12.; (x + (f * 5.).sin() * s * 0.06 + f * s * 0.08, y - f * s) }).collect();
    ribbon(c, &trunk, |t| s * 0.07 * (1. - t * 0.6), Cv::solid(col(tone, 1.)));
    for k in 0..4 {
        let (bx, by) = trunk[4 + k * 2];
        let dir = if k % 2 == 0 { 1f32 } else { -1. };
        let len = s * (0.55 - k as f32 * 0.09);
        let tip = (bx + dir * len, by - s * 0.05);
        ribbon(c, &[(bx, by), ((bx + tip.0) / 2., by + s * 0.02), tip], |t| s * 0.025 * (1. - t * 0.7), Cv::solid(col(tone, 1.)));
        // the needle pad: a flat cloud sitting on the branch
        for j in 0..7 {
            let f = 0.25 + j as f32 * 0.12;
            let (px, py) = (bx + (tip.0 - bx) * f, by - s * 0.03);
            let pw = s * (0.16 - (f - 0.6).abs() * 0.12);
            leaf(c, &[(px - pw, py), (px, py - s * (0.07 + r.f() * 0.03)), (px + pw, py)], s * 0.09, Cv::solid(col(tone, 1.)));
        }
    }
}
fn fill_below(c: &mut Cv, pts: &[(f32, f32)], s: tiny_skia::Shader<'static>) {
    let mut b = PathBuilder::new();
    b.move_to(pts[0].0, c.h);
    for &(x, y) in pts { b.line_to(x, y); }
    b.line_to(pts[pts.len() - 1].0, c.h);
    b.close();
    if let Some(p) = b.finish() { c.fill(&p, s); }
}

/* ── Vagabond (drawn, for when it has no pictures): sumi ink on washi ───────────────────────────────────────────────────────────────────────── */

/// dark washi: a soft lamp-lit centre, long paper fibres, grain
fn washi(c: &mut Cv, t: &Theme, seed: u64, lx: f32, ly: f32) {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    c.fill_all(Cv::radial(w * lx, h * ly, w.max(h) * 0.8, &[(0., col(p.bg.mix(p.surface, 0.9), 1.)), (1., col(p.bg, 1.))]));
    let mut r = Rng::new(seed);
    let u = w.min(h);
    for _ in 0..(w * h / 9000.) as usize {
        let (x, y, a, l) = (r.f() * w, r.f() * h, (r.f() - 0.5) * 0.6, u * (0.01 + r.f() * 0.05));
        let mut b = PathBuilder::new();
        b.move_to(x, y);
        b.quad_to(x + a.cos() * l * 0.5, y + a.sin() * l * 0.5 + (r.f() - 0.5) * l * 0.2, x + a.cos() * l, y + a.sin() * l);
        if let Some(path) = b.finish() { c.stroke(&path, Cv::solid(col(p.ink, 0.025 + r.f() * 0.03)), 0.6 + r.f() * 0.8); }
    }
}
fn seal(c: &mut Cv, t: &Theme, x: f32, y: f32, s: f32) {
    let mut r = Rng::new(17);
    c.fill_rect(x, y, s, s * 1.3, Cv::solid(col(t.palette.accent, 0.95)));
    // the stamp never takes fully: pale flecks in the red
    for _ in 0..40 { c.circle(x + r.f() * s, y + r.f() * s * 1.3, s * (0.01 + r.f() * 0.03), Cv::solid(col(t.palette.bg, 0.35))); }
    c.fill_rect(x + s * 0.18, y + s * 0.2, s * 0.64, s * 0.06, Cv::solid(col(t.palette.bg, 0.55)));
    c.fill_rect(x + s * 0.47, y + s * 0.2, s * 0.06, s * 0.9, Cv::solid(col(t.palette.bg, 0.55)));
}
fn finish(c: &mut Cv, t: &Theme, seed: u32) {
    c.vignette(Rgb(0, 0, 0), 0.55);
    c.noise(0.07, t.palette.ink, seed);
}

fn ronin(c: &mut Cv, t: &Theme, i: usize) {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let mut r = Rng::new(31 + i as u64 * 7);
    match i {
        // 地 Earth: ink-wash mountains under a hazy moon, mist pooling in every valley, a pine on the nearest ridge
        0 => {
            washi(c, t, 5, 0.7, 0.3);
            let (mx, my, mr) = (w * 0.7, h * 0.24, u * 0.085);
            c.cloud(mx, my, mr * 3.5, p.ink, 0.06);
            c.circle(mx, my, mr, Cv::solid(col(p.ink, 0.15)));
            c.fill_all(Cv::linear(0., h * 0.35, 0., h, &[(0., col(p.ink, 0.)), (1., col(p.ink, 0.05))]));
            let mut near = Vec::new();
            for (k, &(base, amp, lift)) in [(0.5f32, 0.26f32, 0.2f32), (0.6, 0.22, 0.13), (0.7, 0.2, 0.07), (0.82, 0.17, 0.)].iter().enumerate() {
                let line = peaks(w, h * base, h * amp, 0.55 + k as f32 * 0.03, &mut r);
                let tone = if lift > 0. { p.bg.mix(p.ink, lift) } else { p.bg.mix(Rgb(0, 0, 0), 0.45) };
                let top = line.iter().map(|q| q.1).fold(h, f32::min);
                // solid at the crest, thinning into the mist below
                fill_below(c, &line, Cv::linear(0., top, 0., h * (base + 0.18), &[(0., col(tone, 1.)), (0.55, col(tone, 0.92)), (1., col(tone, 0.))]));
                brush(c, &line, u * 0.006, p.ink.mix(tone, 0.4), 0.07, &mut r);
                near = line;
            }
            c.fill_rect(0., h * 0.86, w, h * 0.14, Cv::linear(0., h * 0.86, 0., h, &[(0., col(p.bg.mix(Rgb(0, 0, 0), 0.45), 0.)), (1., col(p.bg.mix(Rgb(0, 0, 0), 0.5), 1.))]));
            let (px, py) = near[near.len() * 3 / 10];
            pine(c, px, py + u * 0.01, u * 0.16, p.bg.mix(Rgb(0, 0, 0), 0.6), &mut r);
            seal(c, t, w * 0.9, h * 0.08, u * 0.035);
            finish(c, t, 11);
        }
        // 水 Water: two koi circling under dark water, rings spreading, a lotus pad
        1 => {
            washi(c, t, 6, 0.5, 0.5);
            let (cx, cy) = (w * 0.6, h * 0.52);
            c.cloud(cx, cy, u * 0.5, p.surface.mix(p.accent2, 0.15), 0.35);
            for k in 0..6 {
                let rr = u * (0.08 + k as f32 * 0.07);
                if let Some(path) = Cv::arc(cx + u * 0.02, cy - u * 0.01, rr, rr * 0.42, 0., TAU) {
                    c.stroke(&path, Cv::solid(col(p.ink, 0.10 - k as f32 * 0.014)), u * 0.0025);
                }
            }
            koi(c, cx, cy, u * 0.24, 0.3, p.accent2, p.ink, &mut r);
            koi(c, cx, cy, u * 0.24, 0.3 + PI, p.ink, p.accent, &mut r);
            // petals drifting on the surface
            for _ in 0..9 {
                let (x, y) = (cx + (r.f() - 0.5) * u * 1.1, cy + (r.f() - 0.5) * u * 0.5);
                if let Some(e) = Cv::ellipse(x, y, u * 0.009, u * 0.005, r.f() * PI) { c.fill(&e, Cv::solid(col(p.accent.mix(p.ink, 0.55), 0.7))); }
            }
            // lotus pad with a notch
            let (lx, ly, lr) = (w * 0.2, h * 0.74, u * 0.09);
            if let Some(pad) = Cv::arc(lx, ly, lr, lr * 0.45, 0.25, TAU - 0.05) {
                let mut b = PathBuilder::new();
                b.move_to(lx, ly);
                for pt in pad.points() { b.line_to(pt.x, pt.y); }
                b.close();
                if let Some(pp) = b.finish() { c.fill(&pp, Cv::solid(col(p.bg.mix(p.muted, 0.35), 0.9))); }
            }
            seal(c, t, w * 0.08, h * 0.08, u * 0.035);
            finish(c, t, 12);
        }
        // 火 Fire: a red sun bleeding into the paper, a blade drawn across it, sparks
        2 => {
            washi(c, t, 7, 0.55, 0.45);
            let (sx, sy, sr) = (w * 0.56, h * 0.44, u * 0.27);
            c.cloud(sx, sy, sr * 1.6, p.accent, 0.18);
            for k in 0..60 {
                let a = 0.55 + (k as f32 / 60.) * 0.4;
                let off = (r.f() - 0.5) * sr * 0.05;
                c.circle(sx + off, sy + off * 0.6, sr * (0.985 + r.f() * 0.03), Cv::solid(col(p.accent, a * 0.03)));
            }
            c.circle(sx, sy, sr, Cv::solid(col(p.accent, 0.9)));
            // the blade: a long tapering line with a light edge
            let (x0, y0, x1, y1) = (w * 0.12, h * 0.62, w * 0.9, h * 0.33);
            let mut b = PathBuilder::new();
            b.move_to(x0, y0); b.line_to(x1, y1); b.line_to(x1 - u * 0.03, y1 + u * 0.022); b.line_to(x0 + u * 0.01, y0 + u * 0.012); b.close();
            if let Some(bl) = b.finish() { c.fill(&bl, Cv::solid(col(p.bg.mix(Rgb(0, 0, 0), 0.5), 1.))); }
            c.line(x0, y0, x1, y1, Cv::solid(col(p.ink, 0.85)), u * 0.0025);
            // tsuba and hilt
            let (gx, gy) = (x0 + (x1 - x0) * 0.08, y0 + (y1 - y0) * 0.08);
            c.circle(gx, gy, u * 0.022, Cv::solid(col(p.accent2, 0.9)));
            c.line(x0 - u * 0.09, y0 + u * 0.05, gx, gy, Cv::solid(col(p.bg.mix(Rgb(0, 0, 0), 0.6), 1.)), u * 0.016);
            for _ in 0..70 {
                let (x, y) = (sx + (r.f() - 0.4) * u * 0.9, sy + (r.f() - 0.2) * u * 0.6);
                c.circle(x, y, u * (0.0015 + r.f() * 0.004), Cv::solid(col(p.accent2.mix(p.ink, r.f() * 0.5), 0.3 + r.f() * 0.6)));
            }
            seal(c, t, w * 0.9, h * 0.8, u * 0.035);
            finish(c, t, 13);
        }
        // 風 Wind: two depths of bamboo leaning one way, leaves streaming off, a few torn loose in the air
        3 => {
            washi(c, t, 8, 0.75, 0.3);
            c.cloud(w * 0.78, h * 0.3, u * 0.35, p.ink, 0.05);
            for (depth, n) in [(0usize, 5usize), (1, 4)] {
                let tone = p.bg.mix(p.ink, if depth == 0 { 0.13 } else { 0.32 });
                for k in 0..n {
                    let x = w * (0.03 + k as f32 * (0.4 / n as f32) + r.f() * 0.04) + depth as f32 * w * 0.04;
                    let lean = u * (0.08 + r.f() * 0.1);
                    let thick = u * if depth == 0 { 0.011 } else { 0.017 } * (0.8 + r.f() * 0.4);
                    let pts: Vec<(f32, f32)> = (0..=30).map(|s| { let f = s as f32 / 30.; (x + lean * f * f, h * (1.02 - f * 1.08)) }).collect();
                    // one culm, then at every node a thin gap of paper and a knot either side
                    ribbon(c, &pts, |_| thick, Cv::solid(col(tone, 1.)));
                    for j in (4..30).step_by(5) {
                        let (nx, ny) = pts[j];
                        let (ax, ay) = normal(&pts, j);
                        let (hx, hy) = (ax * thick * 0.75, ay * thick * 0.75);
                        c.line(nx - hx, ny - hy, nx + hx, ny + hy, Cv::solid(col(p.bg, 0.9)), u * 0.0028);
                        for sgn in [-1f32, 1.] {
                            let (kx, ky) = (nx + ax * thick * 0.62 * sgn, ny + ay * thick * 0.62 * sgn);
                            c.line(kx, ky - u * 0.004, kx + ax * thick * 0.25 * sgn, ky + u * 0.003, Cv::solid(col(tone, 1.)), u * 0.0035);
                        }
                    }
                    for _ in 0..(if depth == 1 { 16 } else { 9 }) {
                        let (lx, ly) = pts[13 + (r.f() * 17.) as usize];
                        let ang: f32 = -0.1 + r.f() * 0.6;
                        let l = u * (0.06 + r.f() * 0.05);
                        let end = (lx + ang.cos() * l, ly + ang.sin() * l);
                        leaf(c, &[(lx, ly), (lx + ang.cos() * l * 0.5, ly + ang.sin() * l * 0.5 - l * 0.07), end], l * 0.17, Cv::solid(col(tone, 1.)));
                    }
                }
            }
            for _ in 0..6 {
                let y = h * (0.2 + r.f() * 0.5);
                let x0 = w * (0.4 + r.f() * 0.2);
                let pts: Vec<(f32, f32)> = (0..=24).map(|s| { let f = s as f32 / 24.; (x0 + w * 0.5 * f, y - (f * PI).sin() * u * 0.03) }).collect();
                brush(c, &pts, u * 0.003, p.ink, 0.07, &mut r);
            }
            for _ in 0..12 {
                let (x, y) = (w * (0.5 + r.f() * 0.45), h * (0.12 + r.f() * 0.6));
                let ang = r.f() * TAU;
                let l = u * (0.03 + r.f() * 0.03);
                leaf(c, &[(x, y), (x + ang.cos() * l * 0.5, y + ang.sin() * l * 0.5 + l * 0.08), (x + ang.cos() * l, y + ang.sin() * l)], l * 0.18,
                    Cv::solid(col(p.muted.mix(p.ink, 0.3), 0.75)));
            }
            seal(c, t, w * 0.9, h * 0.08, u * 0.035);
            finish(c, t, 14);
        }
        // 空 Void: one ensō, nothing else
        _ => {
            washi(c, t, 9, 0.62, 0.47);
            let (cx, cy, rad) = (w * 0.62, h * 0.47, u * 0.3);
            for _ in 0..110 {
                let a = 0.05 + r.f() * 0.08;
                let a0 = -1.2 + r.f() * 0.15;
                let a1 = a0 + 5.6 - r.f() * 0.9;
                let off = (r.f() - 0.5) * u * 0.035;
                if let Some(path) = Cv::arc(cx, cy, rad + off, rad + off, a0, a1) { c.stroke(&path, Cv::solid(col(p.ink, a)), u * (0.004 + r.f() * 0.012)); }
            }
            // ink pooled where the brush first touched
            c.cloud(cx + (-1.2f32).cos() * rad, cy + (-1.2f32).sin() * rad, u * 0.03, p.ink, 0.3);
            seal(c, t, w * 0.86, h * 0.76, u * 0.05);
            finish(c, t, 15);
        }
    }
}

/// a koi seen from above, swimming round (cx, cy) at angle a
fn koi(c: &mut Cv, cx: f32, cy: f32, orbit: f32, a: f32, body: Rgb, mark: Rgb, r: &mut Rng) {
    let (x, y) = (cx + a.cos() * orbit, cy + a.sin() * orbit * 0.42);
    let dir = a + PI / 2.;
    let l = orbit * 0.72;
    let tf = Transform::from_rotate(dir.to_degrees()).post_translate(x, y);
    let mut b = PathBuilder::new();
    b.move_to(l * 0.5, 0.);
    b.cubic_to(l * 0.35, -l * 0.16, -l * 0.15, -l * 0.14, -l * 0.42, -l * 0.02);
    b.line_to(-l * 0.62, -l * 0.14); b.line_to(-l * 0.56, 0.); b.line_to(-l * 0.62, l * 0.14); b.line_to(-l * 0.42, l * 0.02);
    b.cubic_to(-l * 0.15, l * 0.14, l * 0.35, l * 0.16, l * 0.5, 0.);
    b.close();
    if let Some(p) = b.finish().and_then(|p| p.transform(tf)) {
        // its shadow on the pond floor
        if let Some(sh) = p.clone().transform(Transform::from_translate(l * 0.07, l * 0.1)) { c.fill(&sh, Cv::solid(col(Rgb(0, 0, 0), 0.35))); }
        c.fill(&p, Cv::solid(col(body, 0.92)));
        // markings, clipped to the body
        c.clip_path(&p);
        for _ in 0..3 {
            let (mx, my) = (l * (r.f() - 0.5) * 0.6, l * (r.f() - 0.5) * 0.15);
            let mut pt = tiny_skia::Point::from_xy(mx, my);
            tf.map_point(&mut pt);
            c.circle(pt.x, pt.y,l * (0.08 + r.f() * 0.08), Cv::solid(col(mark, 0.85)));
        }
        c.clip = None;
        let prev = c.blend;
        c.blend = BlendMode::SourceOver;
        c.stroke(&p, Cv::solid(col(Rgb(0, 0, 0), 0.25)), l * 0.01);
        c.blend = prev;
    }
    // pectoral fins
    for s in [-1f32, 1.] {
        if let Some(f) = crate::paint::Cv::ellipse(0., 0., l * 0.1, l * 0.04, s * 0.7).and_then(|f| f.transform(Transform::from_translate(l * 0.18, s * l * 0.13).post_concat(tf))) {
            c.fill(&f, Cv::solid(col(body, 0.6)));
        }
    }
}

/* ── Fresco: a gilded baroque heaven, seen from below ───────────────────────────────────────────────────── */

const WHITE: Rgb = Rgb(255, 255, 255);
const BLACK: Rgb = Rgb(0, 0, 0);
/// burnished gold across a span: deep, warm, a hot highlight, warm, deep
fn gold(x0: f32, y0: f32, x1: f32, y1: f32, g: Rgb, a: f32) -> tiny_skia::Shader<'static> {
    Cv::linear(x0, y0, x1, y1, &[(0., col(g.mix(BLACK, 0.55), a)), (0.3, col(g, a)), (0.48, col(g.mix(WHITE, 0.6), a)),
        (0.62, col(g, a)), (1., col(g.mix(BLACK, 0.5), a))])
}
/// a heaped cumulus on a flat base at y, lit from `light` (a unit direction), shade in its belly
#[allow(clippy::too_many_arguments)]
fn cumulus(c: &mut Cv, x: f32, y: f32, wid: f32, hgt: f32, lit: Rgb, shade: Rgb, light: (f32, f32), a: f32, r: &mut Rng) {
    // many soft puffs under a dome-shaped envelope over a flat base: painted volume, no hard outline
    let n = ((wid / hgt) * 16.) as usize + 24;
    let mut puffs = Vec::with_capacity(n);
    for _ in 0..n {
        let f = 0.04 + r.f() * 0.92;
        let env = (PI * f).sin().max(0.).powf(0.6);
        let rad = hgt * (0.1 + 0.2 * env) * (0.6 + r.f() * 0.7);
        let py = y - rad * 0.7 - r.f().powf(0.8) * (hgt * env - rad).max(0.);
        puffs.push((x - wid / 2. + wid * f, py, rad));
    }
    puffs.sort_by(|p, q| p.1.partial_cmp(&q.1).unwrap_or(std::cmp::Ordering::Equal));
    let (mx, my) = (x, y - hgt * 0.45);
    // the body in shade, built up softly
    for &(px, py, rad) in &puffs {
        c.circle(px, py, rad, Cv::radial(px, py, rad, &[(0., col(shade, a * 0.92)), (0.72, col(shade, a * 0.8)), (1., col(shade, 0.))]));
    }
    // light on every puff, stronger on the side of the cloud that faces it
    for &(px, py, rad) in &puffs {
        let k = (((px - mx) * light.0 / (wid * 0.5) + (py - my) * light.1 / (hgt * 0.5)) * 0.5 + 0.5).clamp(0., 1.);
        let k = ((k - 0.35) / 0.65).clamp(0., 1.).powf(1.4);   // most of the cloud in shade, the near side lit
        let (lx, ly) = (px + light.0 * rad * 0.35, py + light.1 * rad * 0.35);
        c.circle(lx, ly, rad * 0.8, Cv::radial(lx, ly, rad * 0.8, &[(0., col(lit, a * (0.12 + 0.85 * k))), (0.62, col(lit, a * (0.08 + 0.7 * k))), (1., col(lit, 0.))]));
        if k > 0.75 && r.f() < 0.5 {
            // a sunlit rim
            let (hx, hy) = (px + light.0 * rad * 0.6, py + light.1 * rad * 0.6);
            c.circle(hx, hy, rad * 0.4, Cv::radial(hx, hy, rad * 0.4, &[(0., col(lit.mix(WHITE, 0.4), a * 0.7)), (1., col(lit, 0.))]));
        }
    }
}
/// god-rays fanning from (cx, cy) between angles a0 and a1, added light
#[allow(clippy::too_many_arguments)]
fn rays(c: &mut Cv, cx: f32, cy: f32, n: usize, len: f32, a0: f32, a1: f32, tone: Rgb, a: f32, r: &mut Rng) {
    c.blend = BlendMode::Screen;
    for k in 0..n {
        let ang = a0 + (a1 - a0) * (k as f32 + r.f() * 0.7) / n as f32;
        let wdt = 0.008 + r.f() * 0.03;
        let mut b = PathBuilder::new();
        b.move_to(cx, cy);
        b.line_to(cx + (ang - wdt).cos() * len, cy + (ang - wdt).sin() * len);
        b.line_to(cx + (ang + wdt).cos() * len, cy + (ang + wdt).sin() * len);
        b.close();
        if let Some(p) = b.finish() { c.fill(&p, Cv::radial(cx, cy, len, &[(0., col(tone, a * (0.35 + r.f() * 0.65))), (1., col(tone, 0.))])); }
    }
    c.blend = BlendMode::SourceOver;
}
fn star(c: &mut Cv, x: f32, y: f32, rad: f32, points: usize, s: tiny_skia::Shader<'static>) {
    let mut b = PathBuilder::new();
    for k in 0..points * 2 {
        let rr = if k % 2 == 0 { rad } else { rad * 0.4 };
        let ang = -PI / 2. + k as f32 * PI / points as f32;
        if k == 0 { b.move_to(x + ang.cos() * rr, y + ang.sin() * rr) } else { b.line_to(x + ang.cos() * rr, y + ang.sin() * rr) }
    }
    b.close();
    if let Some(p) = b.finish() { c.fill(&p, s); }
}
/// a turned stone baluster standing on (0,0), rising toward -y, `hh` tall; drawn through c.tf
fn baluster(c: &mut Cv, hh: f32, stone: Rgb, a: f32) {
    // (height, half-width) from foot to top: plinth, belly, neck, collar, cap
    let prof = [(0., 0.3), (0.1, 0.3), (0.12, 0.2), (0.2, 0.17), (0.38, 0.33), (0.52, 0.27), (0.64, 0.12), (0.72, 0.1),
        (0.78, 0.17), (0.84, 0.11), (0.88, 0.29), (1., 0.29)];
    let mut b = PathBuilder::new();
    for (k, &(t, hw)) in prof.iter().enumerate() { let (x, y) = (hw * hh * 0.5, -t * hh); if k == 0 { b.move_to(x, y) } else { b.line_to(x, y) } }
    for &(t, hw) in prof.iter().rev() { b.line_to(-hw * hh * 0.5, -t * hh); }
    b.close();
    let half = hh * 0.17;
    if let Some(p) = b.finish() {
        c.fill(&p, Cv::linear(-half, 0., half, 0., &[(0., col(stone.mix(BLACK, 0.45), a)), (0.35, col(stone.mix(WHITE, 0.25), a)), (1., col(stone.mix(BLACK, 0.35), a))]));
    }
}
/// a gilt rosette: petals round a boss
fn rosette(c: &mut Cv, x: f32, y: f32, rad: f32, g: Rgb) {
    c.circle(x, y, rad * 1.1, Cv::solid(col(BLACK, 0.18)));
    for k in 0..8 {
        let ang = k as f32 * TAU / 8.;
        if let Some(e) = Cv::ellipse(x + ang.cos() * rad * 0.55, y + ang.sin() * rad * 0.55, rad * 0.42, rad * 0.2, ang) {
            c.fill(&e, gold(x - rad, y - rad, x + rad, y + rad, g, 1.));
        }
    }
    c.circle(x, y, rad * 0.3, gold(x - rad * 0.3, y - rad * 0.3, x + rad * 0.3, y + rad * 0.3, g, 1.));
}
/// a gilt moulding around an oval: rim, egg-and-dart, a bright fillet, a bead row
fn gilt_oval(c: &mut Cv, cx: f32, cy: f32, rx: f32, ry: f32, t: f32, g: Rgb) {
    let ring = |c: &mut Cv, d: f32, wdt: f32, s: tiny_skia::Shader<'static>| {
        if let Some(e) = Cv::ellipse(cx, cy, rx + d, ry + d, 0.) { c.stroke(&e, s, wdt); }
    };
    ring(c, t * 0.55, t * 1.5, Cv::solid(col(BLACK, 0.3)));
    ring(c, t * 0.5, t, gold(cx - rx, cy - ry, cx + rx, cy + ry, g, 1.));
    ring(c, t * 0.05, t * 0.12, Cv::solid(col(g.mix(WHITE, 0.55), 1.)));
    ring(c, t * 0.95, t * 0.1, Cv::solid(col(g.mix(BLACK, 0.6), 1.)));
    // egg-and-dart round the band
    let n = ((rx + ry) * PI / (t * 0.9)) as usize;
    for k in 0..n {
        let ang = k as f32 * TAU / n as f32;
        let (x, y) = (cx + ang.cos() * (rx + t * 0.5), cy + ang.sin() * (ry + t * 0.5));
        if let Some(e) = Cv::ellipse(x, y, t * 0.3, t * 0.2, ang) {
            c.fill(&e, Cv::radial(x - t * 0.08, y - t * 0.08, t * 0.35, &[(0., col(g.mix(WHITE, 0.5), 1.)), (1., col(g.mix(BLACK, 0.35), 1.))]));
        }
        let ang2 = ang + PI / n as f32;
        let (dx, dy) = (cx + ang2.cos() * (rx + t * 0.5), cy + ang2.sin() * (ry + t * 0.5));
        c.line(dx - ang2.cos() * t * 0.3, dy - ang2.sin() * t * 0.3, dx + ang2.cos() * t * 0.3, dy + ang2.sin() * t * 0.3, Cv::solid(col(g.mix(BLACK, 0.55), 1.)), t * 0.06);
    }
    // beads inside the rim
    let nb = ((rx + ry) * PI / (t * 0.32)) as usize;
    for k in 0..nb {
        let ang = k as f32 * TAU / nb as f32;
        let (x, y) = (cx + ang.cos() * (rx - t * 0.12), cy + ang.sin() * (ry - t * 0.12));
        c.circle(x, y, t * 0.09, Cv::solid(col(g.mix(WHITE, 0.3), 1.)));
    }
}
/// a painted sky: zenith colour at the top, horizon colour at the bottom
fn sky(c: &mut Cv, top: Rgb, mid: Rgb, low: Rgb) {
    let h = c.h;
    c.fill_all(Cv::linear(0., 0., 0., h, &[(0., col(top, 1.)), (0.55, col(mid, 1.)), (1., col(low, 1.))]));
}
/// a balustrade along the bottom edge, as if the ceiling's painted architecture rose from the room's walls
fn balustrade(c: &mut Cv, y: f32, hh: f32, stone: Rgb, a: f32) {
    let w = c.w;
    let step = hh * 0.62;
    let mut x = step * 0.5;
    while x < w {
        c.tf = Transform::from_translate(x, y);
        baluster(c, hh, stone, a);
        c.tf = Transform::identity();
        x += step;
    }
    // the rail: a cap stone with a lit upper edge
    c.fill_rect(0., y - hh * 1.16, w, hh * 0.16, Cv::linear(0., y - hh * 1.16, 0., y - hh, &[(0., col(stone.mix(WHITE, 0.3), a)), (1., col(stone.mix(BLACK, 0.4), a))]));
    c.fill_rect(0., y - hh * 1.19, w, hh * 0.03, Cv::solid(col(stone.mix(WHITE, 0.5), a)));
}
fn fresco_finish(c: &mut Cv, t: &Theme, seed: u32) {
    // dry plaster under the paint, then the room's dimness at the edges
    c.noise(0.06, t.palette.surface, seed);
    c.vignette(t.palette.bg, 0.5);
}

fn fresco(c: &mut Cv, t: &Theme, i: usize) {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let mut r = Rng::new(53 + i as u64 * 11);
    let (gilt, azure, cream, umber, rose) = (p.accent, p.accent2, p.surface, p.bg, p.muted.mix(p.surface, 0.55));
    let deep = azure.mix(Rgb(20, 30, 70), 0.55);
    match i {
        // I Aurora: dawn breaking over a painted balustrade, rosy clouds lit from below, rays from the horizon
        0 => {
            sky(c, deep, azure.mix(rose, 0.35), rose.mix(gilt, 0.5).mix(WHITE, 0.2));
            let (sx, sy) = (w * 0.3, h * 0.98);
            c.cloud(sx, sy, u * 0.9, gilt.mix(WHITE, 0.5), 0.55);
            rays(c, sx, sy, 34, u * 1.6, -PI * 0.95, -PI * 0.12, gilt.mix(WHITE, 0.6), 0.14, &mut r);
            for &(x, y, s) in &[(0.72f32, 0.32f32, 1.1f32), (0.18, 0.5, 0.9), (0.55, 0.62, 1.3), (0.9, 0.7, 0.8), (0.38, 0.22, 0.6)] {
                let lit = rose.mix(gilt.mix(WHITE, 0.4), 0.5);
                cumulus(c, w * x, h * y, u * 0.55 * s, u * 0.2 * s, lit, azure.mix(umber, 0.35).mix(rose, 0.25), (-0.4, 0.9), 0.95, &mut r);
            }
            balustrade(c, h * 1.0, u * 0.16, cream.mix(rose, 0.25).mix(BLACK, 0.15), 1.);
            fresco_finish(c, t, 3);
        }
        // II Zenith: straight up through an oculus: sky, clouds, a ring of balusters, gilt and stucco round it
        1 => {
            let (cx, cy, rx, ry) = (w * 0.5, h * 0.5, u * 0.44 * (w / h).sqrt(), u * 0.38);
            // stucco ceiling with gilt ribs to the corners and rosettes
            c.fill_all(Cv::radial(cx, cy, w.max(h) * 0.75, &[(0., col(cream, 1.)), (1., col(cream.mix(umber, 0.45), 1.))]));
            for &(ex, ey) in &[(0f32, 0f32), (w, 0.), (0., h), (w, h), (cx, 0.), (cx, h), (0., cy), (w, cy)] {
                let wdt = u * 0.022;
                c.line(cx, cy, ex, ey, Cv::solid(col(BLACK, 0.18)), wdt * 1.6);
                c.line(cx, cy, ex, ey, gold(cx, cy, ex, ey, gilt, 1.), wdt);
            }
            for &(x, y) in &[(0.08f32, 0.12f32), (0.92, 0.12), (0.08, 0.88), (0.92, 0.88)] { rosette(c, w * x, h * y, u * 0.06, gilt); }
            if let Some(o) = Cv::ellipse(cx, cy, rx, ry, 0.) {
                c.clip_path(&o);
                c.fill_all(Cv::radial(cx, cy, rx, &[(0., col(azure.mix(WHITE, 0.25), 1.)), (1., col(deep, 1.))]));
                rays(c, cx, cy, 30, rx * 1.2, 0., TAU, WHITE, 0.08, &mut r);
                for k in 0..5 {
                    let ang = k as f32 * TAU / 5. + 0.4;
                    let d = rx * (0.25 + r.f() * 0.2);
                    cumulus(c, cx + ang.cos() * d, cy + ang.sin() * d * ry / rx, u * 0.3, u * 0.11, cream.mix(WHITE, 0.4), azure.mix(umber, 0.25), (0., -1.), 0.92, &mut r);
                }
                // balusters round the rim, feet on the frame, heads toward the sky
                let n = 34;
                for k in 0..n {
                    let ang = k as f32 * TAU / n as f32;
                    let (x, y) = (cx + ang.cos() * rx, cy + ang.sin() * ry);
                    c.tf = Transform::from_rotate((ang + PI / 2.).to_degrees() + 180.).post_translate(x, y);
                    baluster(c, u * 0.1, cream.mix(umber, 0.15), 1.);
                    c.tf = Transform::identity();
                }
                if let Some(rail) = Cv::ellipse(cx, cy, rx - u * 0.105, ry - u * 0.105, 0.) {
                    c.stroke(&rail, Cv::solid(col(cream.mix(WHITE, 0.2), 1.)), u * 0.018);
                    c.stroke(&rail, Cv::solid(col(BLACK, 0.2)), u * 0.004);
                }
                c.clip = None;
            }
            gilt_oval(c, cx, cy, rx, ry, u * 0.045, gilt);
            fresco_finish(c, t, 5);
        }
        // III Empyrean: the glory: rings of cloud opening onto gold light
        2 => {
            let (cx, cy) = (w * 0.5, h * 0.42);
            c.fill_all(Cv::radial(cx, cy, w.max(h) * 0.8, &[(0., col(gilt.mix(WHITE, 0.85), 1.)), (0.18, col(gilt.mix(WHITE, 0.4), 1.)),
                (0.5, col(gilt.mix(umber, 0.3), 1.)), (1., col(umber.mix(BLACK, 0.2), 1.))]));
            rays(c, cx, cy, 64, w.max(h) * 0.9, 0., TAU, gilt.mix(WHITE, 0.7), 0.16, &mut r);
            // three rings of cloud, darker and nearer outward
            for (ring, &(rad, tone, size)) in [(0.62f32, 0.2f32, 1.25f32), (0.42, 0.45, 0.95), (0.26, 0.7, 0.7)].iter().enumerate() {
                let n = 9 + ring * 3;
                for k in 0..n {
                    let ang = k as f32 * TAU / n as f32 + r.f() * 0.3 + ring as f32;
                    let (x, y) = (cx + ang.cos() * u * rad * 1.5, cy + ang.sin() * u * rad);
                    let lit = gilt.mix(WHITE, 0.3 + tone * 0.4);
                    let shade = umber.mix(gilt, tone * 0.6).mix(cream, 0.15);
                    let (dx, dy) = (cx - x, cy - y);
                    let l = (dx * dx + dy * dy).sqrt().max(1.);
                    cumulus(c, x, y, u * 0.34 * size, u * 0.13 * size, lit, shade, (dx / l, dy / l), 0.95, &mut r);
                }
            }
            c.cloud(cx, cy, u * 0.18, WHITE, 0.6);
            fresco_finish(c, t, 7);
        }
        // IV Vesper: dusk, violet clouds with burning gold rims, the evening star over the balustrade
        3 => {
            let dusk = deep.mix(rose, 0.35).mix(Rgb(70, 40, 90), 0.3);
            sky(c, deep.mix(BLACK, 0.3), dusk, gilt.mix(p.muted, 0.45).mix(WHITE, 0.15));
            let (sx, sy) = (w * 0.74, h * 1.02);
            c.cloud(sx, sy, u * 0.8, gilt.mix(p.muted, 0.3), 0.6);
            rays(c, sx, sy, 16, u * 1.5, -PI * 0.92, -PI * 0.2, gilt.mix(WHITE, 0.3), 0.18, &mut r);
            for &(x, y, s) in &[(0.25f32, 0.3f32, 1.2f32), (0.6, 0.45, 1.0), (0.12, 0.62, 0.8), (0.85, 0.28, 0.7), (0.45, 0.7, 1.1)] {
                cumulus(c, w * x, h * y, u * 0.6 * s, u * 0.18 * s, gilt.mix(p.muted, 0.25).mix(WHITE, 0.15), dusk.mix(BLACK, 0.15), (0.45, 0.9), 0.96, &mut r);
            }
            c.cloud(w * 0.82, h * 0.13, u * 0.05, WHITE, 0.6);
            star(c, w * 0.82, h * 0.13, u * 0.022, 4, Cv::solid(col(WHITE, 0.95)));
            balustrade(c, h * 1.0, u * 0.16, umber.mix(BLACK, 0.35), 1.);
            fresco_finish(c, t, 9);
        }
        // V Nocturne: a blue vault strewn with gilt stars, gilt ribs crossing, a crescent in a medallion
        _ => {
            let vault = azure.mix(Rgb(16, 24, 92), 0.7);
            c.fill_all(Cv::radial(w * 0.5, h * 0.45, w.max(h) * 0.75, &[(0., col(vault.mix(WHITE, 0.08), 1.)), (1., col(vault.mix(BLACK, 0.45), 1.))]));
            let step = u * 0.085;
            let (mut y, mut row) = (step * 0.5, 0);
            while y < h + step {
                let mut x = if row % 2 == 0 { step * 0.5 } else { step };
                while x < w + step {
                    let s = step * (0.16 + r.f() * 0.04);
                    star(c, x, y, s, 8, Cv::radial(x - s * 0.3, y - s * 0.3, s * 1.4, &[(0., col(gilt.mix(WHITE, 0.55), 1.)), (1., col(gilt.mix(BLACK, 0.3), 1.))]));
                    x += step;
                }
                y += step * 0.87; row += 1;
            }
            // the groin vault's ribs: two diagonals and their shadows
            for &(x0, y0, x1, y1) in &[(0f32, 0f32, w, h), (w, 0., 0., h)] {
                let wdt = u * 0.045;
                let l = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
                let (nx, ny) = (-(y1 - y0) / l * wdt / 2., (x1 - x0) / l * wdt / 2.);
                c.line(x0, y0 + wdt * 0.4, x1, y1 + wdt * 0.4, Cv::solid(col(BLACK, 0.35)), wdt * 1.4);
                c.line(x0, y0, x1, y1, gold(x0 - nx, y0 - ny, x0 + nx, y0 + ny, gilt, 1.), wdt);
                c.line(x0, y0, x1, y1, Cv::solid(col(umber, 0.5)), wdt * 0.25);
            }
            let (cx, cy, mr) = (w * 0.5, h * 0.5, u * 0.15);
            let ground = || Cv::radial(cx, cy, mr, &[(0., col(vault.mix(WHITE, 0.12), 1.)), (1., col(vault.mix(BLACK, 0.3), 1.))]);
            c.circle(cx, cy, mr, ground());
            // the crescent: a gold disc with a vault-coloured bite
            c.circle(cx - mr * 0.1, cy, mr * 0.55, Cv::radial(cx - mr * 0.3, cy - mr * 0.2, mr * 0.7, &[(0., col(gilt.mix(WHITE, 0.6), 1.)), (1., col(gilt, 1.))]));
            c.circle(cx + mr * 0.12, cy - mr * 0.08, mr * 0.47, ground());
            gilt_oval(c, cx, cy, mr, mr, u * 0.028, gilt);
            fresco_finish(c, t, 11);
        }
    }
}

/* ── Relic: a museum archive drawer: torn reproductions, catalogue cards, tape, film grain ──────────────── */

/// the drawer's floor: dark linen board, its weave just visible, a lamp pool off-centre
fn linen(c: &mut Cv, t: &Theme, seed: u64) {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    c.fill_all(Cv::radial(w * 0.6, h * 0.4, w.max(h) * 0.85, &[(0., col(p.bg.mix(p.accent2, 0.3), 1.)), (1., col(p.bg.mix(BLACK, 0.4), 1.))]));
    let mut r = Rng::new(seed);
    let step = (w.min(h) / 340.).max(1.5);
    let mut y = 0.;
    while y < h { c.line(0., y, w, y, Cv::solid(col(BLACK, 0.05 + r.f() * 0.06)), step * 0.45); y += step; }
    let mut x = 0.;
    while x < w { c.line(x, 0., x, h, Cv::solid(col(p.surface, 0.012 + r.f() * 0.02)), step * 0.4); x += step; }
}
/// the darker of ink and ground: what prints on cream stock whatever the palette
fn press(t: &Theme) -> Rgb { let p = t.palette; if p.ink.luma() <= p.bg.luma() { p.ink } else { p.bg } }
/// a torn sheet's outline round a w×h rectangle at the origin: deckled all round, one corner torn away if `bite`
fn torn(w: f32, h: f32, rough: f32, bite: Option<usize>, r: &mut Rng) -> Option<tiny_skia::Path> {
    let cs = [(0f32, 0f32), (w, 0.), (w, h), (0., h)];
    let s = w.min(h) * (0.12 + r.f() * 0.08);
    let mut v = Vec::new();
    for (k, &(x, y)) in cs.iter().enumerate() {
        if bite == Some(k) {
            for &(ax, ay) in &[cs[(k + 3) % 4], cs[(k + 1) % 4]] {
                let l = ((ax - x).powi(2) + (ay - y).powi(2)).sqrt();
                let f = s * (0.6 + r.f() * 0.8);
                v.push((x + (ax - x) / l * f, y + (ay - y) / l * f));
            }
        } else { v.push((x, y)); }
    }
    let mut b = PathBuilder::new();
    let mut d = 0f32;
    for k in 0..v.len() {
        let ((x0, y0), (x1, y1)) = (v[k], v[(k + 1) % v.len()]);
        let l = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt().max(1.);
        let n = (l / (rough * 1.5)).max(3.) as usize;
        let (nx, ny) = (-(y1 - y0) / l, (x1 - x0) / l);
        for j in 0..n {
            let f = j as f32 / n as f32;
            d = (d * 0.55 + (r.f() - 0.5) * rough * 1.4).clamp(-rough, rough);
            let (x, y) = (x0 + (x1 - x0) * f + nx * d, y0 + (y1 - y0) * f + ny * d);
            if k == 0 && j == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
        }
    }
    b.close();
    b.finish()
}
/// a reproduction torn from a book and laid in the drawer: its shadow, the paper and margin, the plate itself
#[allow(clippy::too_many_arguments)]
fn print(c: &mut Cv, t: &Theme, cx: f32, cy: f32, pw: f32, ph: f32, rot: f32, margin: f32, bite: Option<usize>, seed: u64, paint: impl Fn(&mut Cv)) {
    let p = t.palette;
    let mut r = Rng::new(seed);
    let rough = pw.min(ph) * 0.005;
    let Some(edge) = torn(pw, ph, rough, bite, &mut r) else { return };
    let place = |dx: f32, dy: f32| Transform::from_translate(-pw / 2. + dx, -ph / 2. + dy).post_rotate(rot.to_degrees()).post_translate(cx, cy);
    for k in 0..7 { let o = (k as f32 + 1.) * pw.min(ph) * 0.004; c.tf = place(o * 0.5, o); c.fill(&edge, Cv::solid(col(BLACK, 0.06))); }
    c.tf = place(0., 0.);
    c.fill(&edge, Cv::linear(0., 0., pw, ph, &[(0., col(p.surface.mix(WHITE, 0.25), 1.)), (1., col(p.surface.mix(p.accent2, 0.15), 1.))]));
    let (iw, ih) = ((pw - margin * 2.).max(4.), (ph - margin * 2.).max(4.));
    let mut plate = Cv::new(iw as u32, ih as u32);
    paint(&mut plate);
    plate.reset();
    c.clip_path(&edge);
    c.tf = place(margin, margin);
    c.draw_image(&plate.px, 0., 0., iw, ih, true);
    c.clip = None;
    // the torn edge: paper fibres catch the light, the cut margin keeps a faint plate line
    c.tf = place(0., 0.);
    c.stroke(&edge, Cv::solid(col(WHITE, 0.35)), rough * 0.9);
    c.fill_rect(margin - 1., margin - 1., iw + 2., 1., Cv::solid(col(BLACK, 0.25)));
    c.tf = Transform::identity();
}
/// masking tape: translucent buff, torn ends, a few crinkles catching light
#[allow(clippy::too_many_arguments)]
fn tape(c: &mut Cv, t: &Theme, x: f32, y: f32, len: f32, wid: f32, rot: f32, seed: u64) {
    let tone = t.palette.surface.mix(Rgb(214, 186, 120), 0.55);
    let mut r = Rng::new(seed);
    let mut b = PathBuilder::new();
    b.move_to(-len / 2., -wid / 2.);
    b.line_to(len / 2., -wid / 2.);
    for k in 1..=8 { b.line_to(len / 2. + (r.f() - 0.5) * wid * 0.14, -wid / 2. + wid * k as f32 / 8.); }
    b.line_to(-len / 2., wid / 2.);
    for k in 1..=8 { b.line_to(-len / 2. + (r.f() - 0.5) * wid * 0.14, wid / 2. - wid * k as f32 / 8.); }
    b.close();
    let Some(path) = b.finish() else { return };
    c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(x + wid * 0.04, y + wid * 0.07);
    c.fill(&path, Cv::solid(col(BLACK, 0.16)));
    c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(x, y);
    c.fill(&path, Cv::linear(0., -wid / 2., 0., wid / 2., &[(0., col(tone.mix(WHITE, 0.25), 0.8)), (1., col(tone, 0.74))]));
    for _ in 0..6 {
        let xx = (r.f() - 0.5) * len * 0.9;
        c.line(xx, -wid / 2., xx + (r.f() - 0.5) * wid * 0.5, wid / 2., Cv::solid(col(WHITE, 0.1 + r.f() * 0.1)), 1.);
    }
    c.tf = Transform::identity();
}
/// a catalogue card: cream stock, a title in Fell, a sanguine rule, typed lines beneath
fn card(c: &mut Cv, t: &Theme, x: f32, y: f32, lw: f32, rot: f32, lines: &[&str]) {
    let p = t.palette;
    let ink = press(t);
    let fs = lw * 0.085;
    // the card grows to fit its longest typed line
    let lw = lines[1..].iter().map(|s| crate::paint::measure(s, "Courier Prime", fs * 0.6)).fold(lw, |m, v| m.max(v + lw * 0.14));
    let ts =fs * (lw * 0.86 / crate::paint::measure(lines[0], "IM Fell English", fs).max(1.)).min(1.);
    let lh = fs * 1.6 + (lines.len() as f32 - 1.) * fs * 0.86 + fs * 0.6;
    c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(x, y);
    c.fill_rect(lw * 0.015, lw * 0.025, lw, lh, Cv::solid(col(BLACK, 0.32)));
    c.fill_rect(0., 0., lw, lh, Cv::linear(0., 0., 0., lh, &[(0., col(p.surface.mix(WHITE, 0.4), 1.)), (1., col(p.surface.mix(WHITE, 0.15), 1.))]));
    c.text(lines[0], "IM Fell English", ts, lw * 0.07, fs * 1.15, col(ink, 0.92));
    c.line(lw * 0.07, fs * 1.5, lw * 0.93, fs * 1.5, Cv::solid(col(p.accent2, 0.7)), (lw * 0.004).max(1.));
    for (k, s) in lines[1..].iter().enumerate() {
        c.text(s, "Courier Prime", fs * 0.6, lw * 0.07, fs * 1.55 + fs * 0.86 * (k as f32 + 1.), col(ink, 0.8));
    }
    c.tf = Transform::identity();
}
/// an accession stamp in blue ink: a double oval, broken where the ink didn't take, a number inside
#[allow(clippy::too_many_arguments)]
fn stamp(c: &mut Cv, t: &Theme, x: f32, y: f32, s: f32, rot: f32, top: &str, num: &str, seed: u64) {
    let blue = t.palette.accent.mix(Rgb(20, 40, 90), 0.45);
    let mut r = Rng::new(seed);
    c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(x, y);
    c.blend = BlendMode::Multiply;
    for &(k, wdt) in &[(1f32, 0.06f32), (0.84, 0.022)] {
        let mut a = 0f32;
        while a < TAU {
            let seg = 0.3 + r.f() * 0.9;
            if let Some(arc) = Cv::arc(0., 0., s * k, s * k * 0.62, a, (a + seg).min(TAU)) { c.stroke(&arc, Cv::solid(col(blue, 0.55 + r.f() * 0.3)), s * wdt); }
            a += seg + r.f() * 0.12;
        }
    }
    let tw = crate::paint::measure(top, "Courier Prime", s * 0.2);
    c.text(top, "Courier Prime", s * 0.2, -tw / 2., -s * 0.1, col(blue, 0.75));
    let nw = crate::paint::measure(num, "Courier Prime", s * 0.3);
    c.text(num, "Courier Prime", s * 0.3, -nw / 2., s * 0.24, col(blue, 0.8));
    c.blend = BlendMode::SourceOver;
    c.tf = Transform::identity();
}
/// a length of 35 mm film: dark base, sprocket holes, edge numbers, frames holding a faint amber image
#[allow(clippy::too_many_arguments)]
fn film(c: &mut Cv, t: &Theme, x: f32, y: f32, len: f32, fw: f32, rot: f32, first: usize, seed: u64) {
    let p = t.palette;
    let base = Rgb(28, 19, 12);
    let amber = Rgb(214, 150, 70);
    let mut r = Rng::new(seed);
    c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(x, y);
    c.fill_rect(fw * 0.05, -fw / 2. + fw * 0.08, len, fw, Cv::solid(col(BLACK, 0.4)));
    c.fill_rect(0., -fw / 2., len, fw, Cv::linear(0., -fw / 2., 0., fw / 2., &[(0., col(base.mix(amber, 0.18), 0.97)), (0.5, col(base, 0.97)), (1., col(base.mix(amber, 0.14), 0.97))]));
    let (hole, pitch) = (fw * 0.075, fw * 0.136);
    let mut s = pitch * 0.4;
    while s < len - hole {
        for &yy in &[-fw / 2. + fw * 0.1, fw / 2. - fw * 0.1 - hole * 1.3] {
            c.fill_rect(s, yy, hole, hole * 1.3, Cv::solid(col(p.bg.mix(BLACK, 0.3), 1.)));
        }
        s += pitch;
    }
    let (fh, gap) = (fw * 0.6, fw * 0.07);
    let fwid = fh * 1.45;
    let mut fx = gap;
    let mut n = first;
    while fx + fwid < len {
        let (bx, by) = (fx + fwid * (0.25 + r.f() * 0.5), -fh * (0.2 - r.f() * 0.4));
        let lum = 0.25 + r.f() * 0.5;
        c.fill_rect(fx, -fh / 2., fwid, fh, Cv::radial(bx, by, fh, &[(0., col(amber.mix(WHITE, lum), 0.55)), (0.6, col(amber.mix(BLACK, 0.7 - lum * 0.5), 0.5)), (1., col(base, 0.6))]));
        // what each frame caught: a horizon, a figure, or nothing but light
        match (r.f() * 3.) as usize {
            0 => c.fill_rect(fx, fh * (0.05 + r.f() * 0.15), fwid, fh * 0.45 - fh * 0.1, Cv::solid(col(base, 0.45))),
            1 => if let Some(e) = Cv::ellipse(fx + fwid * (0.3 + r.f() * 0.4), fh * 0.12, fh * 0.12, fh * 0.32, 0.) { c.fill(&e, Cv::solid(col(base, 0.5))); },
            _ => {}
        }
        c.text(&format!("{n}"), "Courier Prime", fw * 0.07, fx + fwid * 0.42, fw / 2. - fw * 0.02, col(amber, 0.85));
        c.text(&format!("{n}A"), "Courier Prime", fw * 0.07, fx + fwid * 0.9, -fw / 2. + fw * 0.08, col(amber, 0.6));
        fx += fwid + gap;
        n += 1;
    }
    c.tf = Transform::identity();
}
/// old varnish and craquelure over a painted plate: an amber glaze, a darkened rim, a net of fine cracks
fn age(c: &mut Cv, seed: u64) {
    let (w, h) = (c.w, c.h);
    c.blend = BlendMode::Multiply;
    c.fill_all(Cv::solid(col(Rgb(238, 210, 156), 1.)));
    c.blend = BlendMode::SourceOver;
    c.vignette(BLACK, 0.4);
    let mut r = Rng::new(seed);
    let u = w.min(h);
    for _ in 0..(w * h / 2400.) as usize {
        let (mut x, mut y) = (r.f() * w, r.f() * h);
        let mut a = r.f() * TAU;
        let mut b = PathBuilder::new();
        b.move_to(x, y);
        for _ in 0..(3 + (r.f() * 5.) as usize) { a += (r.f() - 0.5) * 1.6; let l = u * (0.006 + r.f() * 0.014); x += a.cos() * l; y += a.sin() * l; b.line_to(x, y); }
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(BLACK, 0.1 + r.f() * 0.1)), 0.5 + r.f() * 0.4); }
    }
    c.noise(0.06, Rgb(40, 28, 16), seed as u32);
}
/// brushwork: dabs of the colour already there, dragged a little, so the plate reads as paint and not as gradients
fn brushwork(c: &mut Cv, n: usize, len: f32, wid: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let at = |c: &Cv, x: f32, y: f32| c.px.pixel(x.max(0.) as u32, y.max(0.) as u32).map(|p| p.demultiply());
    for _ in 0..n {
        let (x, y) = (r.f() * c.w, r.f() * c.h);
        let a = r.f() * PI;
        let l = len * (0.5 + r.f());
        let (x0, y0, x1, y1) = (x - a.cos() * l / 2., y - a.sin() * l / 2., x + a.cos() * l / 2., y + a.sin() * l / 2.);
        // only within one patch of colour: a stroke that would cross an edge is not laid, so edges stay clean
        let (Some(d), Some(e0), Some(e1)) = (at(c, x, y), at(c, x0, y0), at(c, x1, y1)) else { continue };
        let diff = |p: tiny_skia::ColorU8| (p.red() as i32 - d.red() as i32).abs() + (p.green() as i32 - d.green() as i32).abs() + (p.blue() as i32 - d.blue() as i32).abs();
        if diff(e0) > 36 || diff(e1) > 36 { continue; }
        c.line(x0, y0, x1, y1, Cv::solid(tiny_skia::Color::from_rgba8(d.red(), d.green(), d.blue(), 130)), wid * (0.5 + r.f()));
    }
}
/// a smooth curve through points (Catmull-Rom), `per` steps between each pair
fn smooth(p: &[(f32, f32)], per: usize) -> Vec<(f32, f32)> {
    let n = p.len();
    let mut out = Vec::new();
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (p[i.saturating_sub(1)], p[i], p[i + 1], p[(i + 2).min(n - 1)]);
        for k in 0..per {
            let t = k as f32 / per as f32;
            let f = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2. * b + (-a + c) * t + (2. * a - 5. * b + 4. * c - d) * t * t + (-a + 3. * b - 3. * c + d) * t * t * t);
            out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out.push(p[n - 1]);
    out
}
/// a turned vessel standing on (x, y): a profile of (height, half-width) pairs from the foot up, fractions of hh
fn vessel(c: &mut Cv, x: f32, y: f32, hh: f32, prof: &[(f32, f32)], s: tiny_skia::Shader<'static>) -> Option<tiny_skia::Path> {
    let pts = smooth(prof, 6);
    let mut b = PathBuilder::new();
    for (k, &(t, hw)) in pts.iter().enumerate() { let (px, py) = (x + hw * hh, y - t * hh); if k == 0 { b.move_to(px, py) } else { b.line_to(px, py) } }
    for &(t, hw) in pts.iter().rev() { b.line_to(x - hw * hh, y - t * hh); }
    b.close();
    let p = b.finish()?;
    c.fill(&p, s);
    Some(p)
}

//// I Portraits: a bearded man in profile, a tall black hat and a millstone ruff, lit from the left out of the dark
fn portrait(c: &mut Cv, seed: u64) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    c.fill_all(Cv::radial(w * 0.3, h * 0.34, h * 0.8, &[(0., col(Rgb(124, 96, 60), 1.)), (0.42, col(Rgb(58, 42, 26), 1.)), (1., col(Rgb(16, 11, 7), 1.))]));
    let s = u * 0.36;
    let (hx, hy) = (w * 0.54, h * 0.38);
    let at = |x: f32, y: f32| (hx + x * s, hy + y * s);
    let shape = |pts: &[(f32, f32)]| {
        let sm = smooth(pts, 6);
        let mut b = PathBuilder::new();
        for (k, &(x, y)) in sm.iter().enumerate() { let (px, py) = at(x, y); if k == 0 { b.move_to(px, py) } else { b.line_to(px, py) } }
        b.close();
        b.finish()
    };
    // the doublet: turned three-quarters, the near shoulder broad on the right, a dull sheen where the light reaches
    if let Some(p) = shape(&[(-0.05, 0.8), (-0.6, 0.95), (-0.95, 1.6), (1.6, 1.6), (1.15, 0.95), (0.45, 0.78)]) {
        let (a, b) = (at(-0.9, 0.8), at(1.2, 1.4));
        c.fill(&p, Cv::linear(a.0, a.1, b.0, b.1, &[(0., col(Rgb(76, 58, 40), 1.)), (0.35, col(Rgb(30, 22, 16), 1.)), (1., col(Rgb(10, 7, 5), 1.))]));
    }
    // the head and neck in profile, facing left: brow, nose, lips, chin, the throat, the nape
    let head = shape(&[(0.3, -0.58), (0.02, -0.6), (-0.2, -0.5), (-0.3, -0.32), (-0.33, -0.16), (-0.3, -0.07), (-0.38, 0.04), (-0.47, 0.13), (-0.42, 0.17),
        (-0.35, 0.19), (-0.36, 0.24), (-0.33, 0.27), (-0.35, 0.31), (-0.32, 0.35), (-0.27, 0.39), (-0.29, 0.45), (-0.25, 0.52), (-0.15, 0.58), (-0.06, 0.62),
        (-0.04, 0.75), (-0.03, 0.95), (0.36, 0.95), (0.34, 0.6), (0.42, 0.4), (0.53, 0.12), (0.53, -0.2), (0.46, -0.45), (0.3, -0.58)]);
    if let Some(hp) = &head {
        let (a, b) = (at(-0.5, 0.), at(0.55, 0.));
        c.fill(hp, Cv::linear(a.0, a.1, b.0, b.1, &[(0., col(Rgb(240, 202, 162), 1.)), (0.32, col(Rgb(210, 154, 110), 1.)), (0.62, col(Rgb(132, 86, 56), 1.)), (1., col(Rgb(46, 30, 18), 1.))]));
        c.clip_path(hp);
        // modelling: the lit planes of forehead, cheekbone, nose and chin; the shadow under the jaw and in the temple
        for &(x, y, rad, tone, a) in &[(-0.15f32, -0.36f32, 0.2f32, Rgb(250, 222, 190), 0.5f32), (-0.12, 0.06, 0.14, Rgb(246, 206, 170), 0.45), (-0.36, -0.02, 0.06, Rgb(255, 230, 200), 0.6),
            (-0.4, 0.11, 0.05, Rgb(250, 214, 180), 0.5), (-0.22, 0.46, 0.08, Rgb(236, 190, 150), 0.4), (0.12, 0.34, 0.28, Rgb(40, 24, 14), 0.55),
            (0.05, -0.2, 0.18, Rgb(70, 44, 28), 0.35), (-0.05, 0.16, 0.12, Rgb(190, 100, 80), 0.25), (-0.1, 0.7, 0.2, Rgb(30, 18, 10), 0.6)] {
            let (x, y) = at(x, y);
            c.cloud(x, y, rad * s, tone, a);
        }
        // the eye: socket, the lid's line, the white and a dark iris looking out of the picture, one catchlight
        let (ex, ey) = at(-0.22, -0.07);
        c.cloud(ex + s * 0.03, ey - s * 0.01, s * 0.1, Rgb(80, 46, 28), 0.5);
        if let Some(e) = Cv::ellipse(ex, ey, s * 0.045, s * 0.02, 0.15) { c.fill(&e, Cv::solid(col(Rgb(214, 196, 176), 0.85))); }
        c.circle(ex - s * 0.022, ey + s * 0.002, s * 0.017, Cv::solid(col(Rgb(34, 22, 14), 0.95)));
        c.circle(ex - s * 0.028, ey - s * 0.005, s * 0.005, Cv::solid(col(WHITE, 0.8)));
        let (l0, l1) = (at(-0.275, -0.085), at(-0.18, -0.1));
        c.line(l0.0, l0.1, l1.0, l1.1, Cv::solid(col(Rgb(60, 34, 20), 0.75)), s * 0.012);
        let (b0, b1) = (at(-0.31, -0.16), at(-0.13, -0.19));
        c.line(b0.0, b0.1, b1.0, b1.1, Cv::solid(col(Rgb(70, 46, 30), 0.55)), s * 0.022);
        // nostril, the fold from nose to mouth, the mouth's line and the lip
        let (nx, ny) = at(-0.37, 0.12);
        c.cloud(nx, ny, s * 0.018, Rgb(70, 36, 22), 0.4);
        let (fx, fy) = at(-0.28, 0.18);
        c.cloud(fx, fy, s * 0.05, Rgb(110, 60, 40), 0.4);
        let (m0, m1) = (at(-0.355, 0.272), at(-0.27, 0.285));
        c.line(m0.0, m0.1, m1.0, m1.1, Cv::solid(col(Rgb(80, 34, 26), 0.7)), s * 0.01);
        let (lx, ly) = at(-0.33, 0.3);
        c.cloud(lx, ly, s * 0.03, Rgb(176, 92, 74), 0.5);
        // the hair: a dark mass from the crown down behind the ear to the nape, its front edge warm where the light grazes it
        if let Some(p) = shape(&[(0.04, -0.66), (0.62, -0.66), (0.64, 0.5), (0.36, 0.56), (0.3, 0.3), (0.25, 0.1), (0.22, -0.14), (0.1, -0.32), (0.0, -0.5)]) {
            let (a, b) = at(0.05, 0.);
            c.fill(&p, Cv::linear(a, b, a + s * 0.5, b, &[(0., col(Rgb(96, 66, 42), 1.)), (0.3, col(Rgb(46, 30, 18), 1.)), (1., col(Rgb(16, 10, 6), 1.))]));
            for k in 0..9 {
                let f = k as f32 / 8.;
                let pts = smooth(&[at(0.06 + f * 0.25, -0.55), at(0.12 + f * 0.25, -0.2), at(0.28 + f * 0.2, 0.2), at(0.33 + f * 0.2, 0.5)], 6);
                brush(c, &pts, s * 0.03, Rgb(120, 86, 56), 0.12, &mut Rng::new(seed + k));
            }
        }
        let (ax, ay) = at(0.16, 0.02);
        if let Some(e) = Cv::ellipse(ax, ay, s * 0.07, s * 0.12, 0.2) { c.fill(&e, Cv::radial(ax - s * 0.03, ay - s * 0.03, s * 0.13, &[(0., col(Rgb(206, 146, 104), 1.)), (1., col(Rgb(100, 60, 38), 1.))])); }
        if let Some(e) = Cv::ellipse(ax + s * 0.01, ay, s * 0.03, s * 0.065, 0.2) { c.fill(&e, Cv::solid(col(Rgb(70, 40, 24), 0.6))); }
        c.clip = None;
    }
    // a moustache over the lip and a pointed beard on the chin, the lit skin left between them
    let mo = smooth(&[at(-0.39, 0.21), at(-0.34, 0.2), at(-0.28, 0.222), at(-0.23, 0.26)], 6);
    ribbon(c, &mo, |t| s * 0.04 * (1. - t * 0.6), Cv::solid(col(Rgb(70, 46, 28), 0.95)));
    if let Some(p) = shape(&[(-0.33, 0.345), (-0.27, 0.37), (-0.21, 0.43), (-0.17, 0.53), (-0.21, 0.67), (-0.26, 0.6), (-0.3, 0.5), (-0.31, 0.42)]) {
        let (a, b) = (at(-0.32, 0.36), at(-0.18, 0.66));
        c.fill(&p, Cv::linear(a.0, a.1, b.0, b.1, &[(0., col(Rgb(104, 72, 46), 0.95)), (1., col(Rgb(30, 20, 12), 0.95))]));
    }
    // the hat: a tall black capotain, its brim tilted down to the front
    if let Some(p) = shape(&[(-0.28, -0.5), (-0.2, -0.85), (-0.12, -1.02), (0.25, -1.03), (0.36, -0.86), (0.4, -0.5)]) {
        let (a, b) = (at(-0.3, -0.9), at(0.4, -0.5));
        c.fill(&p, Cv::linear(a.0, a.1, b.0, b.1, &[(0., col(Rgb(64, 52, 42), 1.)), (0.3, col(Rgb(26, 20, 16), 1.)), (1., col(Rgb(8, 6, 5), 1.))]));
    }
    let (bx, by) = at(0.05, -0.5);
    if let Some(e) = Cv::ellipse(bx, by, s * 0.62, s * 0.08, -0.12) {
        c.fill(&e, Cv::linear(bx - s * 0.6, 0., bx + s * 0.6, 0., &[(0., col(Rgb(70, 58, 46), 1.)), (0.4, col(Rgb(22, 17, 13), 1.)), (1., col(Rgb(8, 6, 5), 1.))]));
    }
    let (hb0, hb1) = (at(-0.24, -0.6), at(0.37, -0.62));
    c.line(hb0.0, hb0.1, hb1.0, hb1.1, Cv::solid(col(Rgb(50, 40, 32), 0.9)), s * 0.035);
    // the ruff, seen from the side: a thick millstone of starched linen; its edge a row of rounded pleats, its top in light
    let (fx, fy, frx, fry, th) = (hx + s * 0.15, hy + s * 0.8, s * 0.62, s * 0.19, s * 0.11);
    if let Some(e) = Cv::ellipse(fx, fy + th, frx, fry, 0.) { c.fill(&e, Cv::solid(col(Rgb(70, 64, 58), 1.))); }
    let n = 30;
    let mut pleats: Vec<f32> = (0..=n).map(|k| k as f32 * PI / n as f32).collect();
    pleats.sort_by(|a, b| a.sin().partial_cmp(&b.sin()).unwrap_or(std::cmp::Ordering::Equal));
    for a in pleats {
        let (x, y) = (fx + a.cos() * frx, fy + a.sin() * fry + th * 0.5);
        let lit = 0.35 + 0.65 * (0.5 - a.cos() * 0.5);
        if let Some(e) = Cv::ellipse(x, y, frx * 0.075, th * 0.62, 0.) {
            c.fill(&e, Cv::radial(x - frx * 0.03, y - th * 0.2, th * 0.8, &[(0., col(Rgb(240, 234, 220).mix(Rgb(90, 84, 76), 1. - lit), 1.)), (1., col(Rgb(110, 102, 92).mix(BLACK, 1. - lit), 1.))]));
        }
    }
    if let Some(e) = Cv::ellipse(fx, fy, frx * 1.02, fry, 0.) {
        c.fill(&e, Cv::radial(fx - frx * 0.5, fy - fry * 0.4, frx * 1.3, &[(0., col(Rgb(246, 240, 226), 1.)), (0.55, col(Rgb(196, 188, 172), 1.)), (1., col(Rgb(110, 104, 94), 1.))]));
    }
    for k in 0..n * 3 {
        let a = k as f32 * TAU / (n * 3) as f32;
        let sh = 0.12 + 0.25 * (a.cos() * 0.5 + 0.5);
        c.line(fx + a.cos() * frx * 0.3, fy + a.sin() * fry * 0.3, fx + a.cos() * frx, fy + a.sin() * fry, Cv::solid(col(Rgb(70, 62, 54), sh)), s * 0.005);
    }
    if let Some(e) = Cv::ellipse(fx, fy + th * 0.5, frx * 1.08, fry + th * 0.6, 0.) { c.fill(&e, Cv::linear(fx - frx, 0., fx + frx, 0., &[(0., col(BLACK, 0.)), (0.5, col(BLACK, 0.08)), (1., col(BLACK, 0.5))])); }
    brushwork(c, (w * h / 90.) as usize, u * 0.025, u * 0.007, seed);
    age(c, seed + 1);
}
// II Still Life: a pewter jug, a roemer, grapes and a half-peeled lemon on a stone ledge in the dark
fn still_life(c: &mut Cv, seed: u64) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let mut r = Rng::new(seed);
    c.fill_all(Cv::radial(w * 0.3, h * 0.3, w.max(h) * 0.85, &[(0., col(Rgb(74, 60, 40), 1.)), (0.5, col(Rgb(30, 24, 17), 1.)), (1., col(Rgb(10, 8, 6), 1.))]));
    let ly = h * 0.68;
    c.fill_rect(0., ly, w, h * 0.06, Cv::linear(0., 0., w, 0., &[(0., col(Rgb(156, 134, 100), 1.)), (1., col(Rgb(66, 56, 42), 1.))]));
    c.fill_rect(0., ly + h * 0.06, w, h, Cv::linear(0., ly, 0., h, &[(0., col(Rgb(62, 50, 36), 1.)), (1., col(Rgb(18, 14, 10), 1.))]));
    c.fill_rect(0., ly + h * 0.06, w, h * 0.004, Cv::solid(col(Rgb(190, 170, 130), 0.35)));
    let pewter = |x: f32, hw: f32| Cv::linear(x - hw, 0., x + hw, 0., &[(0., col(Rgb(30, 30, 30), 1.)), (0.22, col(Rgb(120, 118, 110), 1.)), (0.3, col(Rgb(226, 222, 210), 1.)),
        (0.38, col(Rgb(130, 126, 116), 1.)), (0.7, col(Rgb(60, 58, 54), 1.)), (0.86, col(Rgb(96, 92, 84), 1.)), (1., col(Rgb(24, 24, 24), 1.))]);
    // the jug: belly, waist, a domed lid and thumb-piece, a strap handle on the right
    let (jx, jy, jh) = (w * 0.3, ly + h * 0.035, u * 0.5);
    c.cloud(jx + jh * 0.15, jy, jh * 0.4, BLACK, 0.45);
    let handle = smooth(&[(jx + jh * 0.2, jy - jh * 0.78), (jx + jh * 0.42, jy - jh * 0.74), (jx + jh * 0.44, jy - jh * 0.5), (jx + jh * 0.3, jy - jh * 0.28)], 8);
    ribbon(c, &handle, |t| jh * (0.06 - t * 0.02), Cv::linear(jx + jh * 0.2, 0., jx + jh * 0.46, 0., &[(0., col(Rgb(40, 40, 38), 1.)), (0.5, col(Rgb(150, 146, 136), 1.)), (1., col(Rgb(30, 30, 30), 1.))]));
    vessel(c, jx, jy, jh, &[(0., 0.24), (0.04, 0.26), (0.08, 0.22), (0.3, 0.33), (0.5, 0.3), (0.66, 0.2), (0.78, 0.18), (0.84, 0.21), (0.86, 0.2)], pewter(jx, jh * 0.34));
    if let Some(e) = Cv::ellipse(jx, jy - jh * 0.88, jh * 0.2, jh * 0.06, 0.) { c.fill(&e, pewter(jx, jh * 0.2)); }
    // the roemer: green glass, prunts on the stem, pale wine in the bowl, the window caught twice
    let (gx, gy, gh) = (w * 0.56, ly + h * 0.03, u * 0.4);
    let glass = Rgb(110, 150, 90);
    let prof = [(0., 0.2), (0.03, 0.2), (0.07, 0.1), (0.42, 0.11), (0.47, 0.15), (0.5, 0.13), (0.62, 0.24), (0.82, 0.27), (1., 0.22)];
    if let Some(p) = vessel(c, gx, gy, gh, &prof, Cv::solid(col(glass, 0.22))) { c.stroke(&p, Cv::solid(col(glass.mix(WHITE, 0.4), 0.45)), u * 0.003); }
    if let Some(e) = Cv::ellipse(gx, gy - gh * 0.78, gh * 0.26, gh * 0.12, 0.) { c.fill(&e, Cv::solid(col(Rgb(214, 186, 100), 0.35))); }
    if let Some(e) = Cv::ellipse(gx, gy - gh, gh * 0.22, gh * 0.04, 0.) { c.stroke(&e, Cv::solid(col(WHITE, 0.4)), u * 0.003); }
    for k in 0..6 { let y = gy - gh * (0.12 + k as f32 * 0.05); let x = gx + if k % 2 == 0 { -gh * 0.06 } else { gh * 0.06 }; c.circle(x, y, gh * 0.03, Cv::radial(x - gh * 0.01, y - gh * 0.01, gh * 0.04, &[(0., col(glass.mix(WHITE, 0.6), 0.9)), (1., col(glass.mix(BLACK, 0.4), 0.8))])); }
    c.line(gx - gh * 0.12, gy - gh * 0.62, gx - gh * 0.14, gy - gh * 0.92, Cv::solid(col(WHITE, 0.75)), u * 0.006);
    c.line(gx + gh * 0.16, gy - gh * 0.66, gx + gh * 0.17, gy - gh * 0.84, Cv::solid(col(WHITE, 0.35)), u * 0.004);
    // grapes spilling over the ledge's edge
    let (qx, qy, qs) = (w * 0.44, ly + h * 0.02, u * 0.035);
    let counts = [4usize, 5, 4, 4, 3, 2, 1];
    let mut grapes = Vec::new();
    for (row, &n) in counts.iter().enumerate() { for k in 0..n { grapes.push((qx + (k as f32 - (n as f32 - 1.) / 2.) * qs * 1.7 + (r.f() - 0.5) * qs * 0.4, qy + row as f32 * qs * 1.35 + (r.f() - 0.5) * qs * 0.3)); } }
    c.line(qx, qy - qs * 2., qx + qs * 0.6, qy - qs * 0.5, Cv::solid(col(Rgb(70, 54, 30), 1.)), qs * 0.25);
    for &(x, y) in &grapes {
        c.circle(x, y, qs, Cv::radial(x - qs * 0.35, y - qs * 0.35, qs * 1.3, &[(0., col(Rgb(150, 116, 140), 1.)), (0.5, col(Rgb(66, 32, 58), 1.)), (1., col(Rgb(18, 10, 16), 1.))]));
        c.circle(x - qs * 0.35, y - qs * 0.38, qs * 0.14, Cv::solid(col(WHITE, 0.7)));
    }
    // the lemon, its peel hanging in a spiral over the edge
    let (lx, lyy, ls) = (w * 0.72, ly + h * 0.01, u * 0.085);
    c.cloud(lx + ls * 0.3, lyy + ls * 0.6, ls * 1.1, BLACK, 0.4);
    if let Some(e) = Cv::ellipse(lx, lyy, ls, ls * 0.72, -0.2) { c.fill(&e, Cv::radial(lx - ls * 0.4, lyy - ls * 0.35, ls * 1.5, &[(0., col(Rgb(252, 230, 130), 1.)), (0.4, col(Rgb(226, 178, 40), 1.)), (1., col(Rgb(96, 60, 10), 1.))])); }
    c.circle(lx - ls * 1.0, lyy + ls * 0.18, ls * 0.12, Cv::solid(col(Rgb(200, 150, 30), 1.)));
    let peel: Vec<(f32, f32)> = (0..=48).map(|k| { let t = k as f32 / 48.; (lx + ls * 0.6 + (t * TAU * 2.2).sin() * ls * 0.5 * (1. - t * 0.3), lyy + ls * 0.5 + t * h * 0.27) }).collect();
    ribbon(c, &peel, |t| ls * 0.3 * (0.25 + 0.75 * (t * TAU * 2.2).cos().abs()), Cv::linear(lx, 0., lx + ls * 1.2, 0., &[(0., col(Rgb(240, 196, 60), 1.)), (1., col(Rgb(150, 100, 20), 1.))]));
    ribbon(c, &peel, |t| ls * 0.12 * (0.2 + 0.8 * (t * TAU * 2.2).sin().max(0.)), Cv::solid(col(Rgb(246, 236, 200), 0.85)));
    // a knife on the ledge, its handle jutting out over the edge toward us
    let (k0, k1, k2) = ((w * 0.78, ly + h * 0.015), (w * 0.88, ly + h * 0.03), (w * 0.98, ly + h * 0.1));
    c.line(k0.0, k0.1, k1.0, k1.1, Cv::linear(k0.0, k0.1 - 4., k0.0, k0.1 + 4., &[(0., col(Rgb(230, 226, 214), 1.)), (1., col(Rgb(90, 88, 84), 1.))]), u * 0.016);
    c.line(k1.0, k1.1 + h * 0.06, k2.0, k2.1 + h * 0.06, Cv::solid(col(BLACK, 0.35)), u * 0.03);
    c.line(k1.0, k1.1, k2.0, k2.1, Cv::solid(col(Rgb(70, 44, 26), 1.)), u * 0.026);
    c.line(k1.0, k1.1 - u * 0.006, k2.0, k2.1 - u * 0.006, Cv::solid(col(Rgb(180, 130, 80), 0.6)), u * 0.004);
    brushwork(c, (w * h / 110.) as usize, u * 0.025, u * 0.007, seed + 2);
    age(c, seed + 3);
}
/// a post mill: a dark body on its trestle, four lattice sails turned to `a`
fn windmill(c: &mut Cv, x: f32, y: f32, s: f32, a: f32, tone: Rgb) {
    let mut b = PathBuilder::new();
    b.move_to(x - s * 0.2, y);
    b.line_to(x - s * 0.08, y - s * 0.4);
    b.line_to(x + s * 0.08, y - s * 0.4);
    b.line_to(x + s * 0.2, y);
    b.close();
    if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(tone, 1.)), s * 0.03); }
    c.fill_rect(x - s * 0.14, y - s * 0.78, s * 0.28, s * 0.4, Cv::solid(col(tone, 1.)));
    if let Some(e) = Cv::ellipse(x, y - s * 0.78, s * 0.15, s * 0.06, 0.) { c.fill(&e, Cv::solid(col(tone, 1.))); }
    let (hx, hy) = (x - s * 0.02, y - s * 0.66);
    for k in 0..4 {
        let ang = a + k as f32 * PI / 2.;
        let (dx, dy, nx, ny) = (ang.cos(), ang.sin(), -ang.sin(), ang.cos());
        c.line(hx, hy, hx + dx * s * 0.95, hy + dy * s * 0.95, Cv::solid(col(tone, 1.)), s * 0.025);
        for j in 0..=8 {
            let f = 0.2 + j as f32 * 0.09;
            c.line(hx + dx * s * f, hy + dy * s * f, hx + dx * s * f + nx * s * 0.16, hy + dy * s * f + ny * s * 0.16, Cv::solid(col(tone, 0.9)), s * 0.008);
        }
        c.line(hx + dx * s * 0.2 + nx * s * 0.16, hy + dy * s * 0.2 + ny * s * 0.16, hx + dx * s * 0.92 + nx * s * 0.16, hy + dy * s * 0.92 + ny * s * 0.16, Cv::solid(col(tone, 0.9)), s * 0.01);
    }
}
/// III Landscape: a polder under a towering sky, sun breaking through onto the fields, a mill and a far spire
fn landscape(c: &mut Cv, seed: u64) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let mut r = Rng::new(seed);
    let hz = h * 0.74;
    c.fill_all(Cv::linear(0., 0., 0., hz, &[(0., col(Rgb(64, 80, 96), 1.)), (0.6, col(Rgb(150, 154, 144), 1.)), (1., col(Rgb(222, 206, 166), 1.))]));
    for &(x, y, s) in &[(0.22f32, 0.3f32, 1.2f32), (0.7, 0.22, 1.5), (0.5, 0.52, 0.9), (0.9, 0.5, 0.8), (0.1, 0.6, 0.7)] {
        cumulus(c, w * x, h * y, u * 0.75 * s, u * 0.26 * s, Rgb(244, 232, 204), Rgb(104, 108, 106), (-0.45, -0.85), 0.95, &mut r);
    }
    let land = peaks(w, hz + h * 0.01, h * 0.012, 0.6, &mut r);
    fill_below(c, &land, Cv::linear(0., hz, 0., h, &[(0., col(Rgb(70, 72, 52), 1.)), (1., col(Rgb(22, 22, 14), 1.))]));
    // the distance: a pale band, a spire, a row of trees
    c.fill_rect(0., hz - h * 0.004, w, h * 0.01, Cv::solid(col(Rgb(120, 130, 120), 0.6)));
    let (sx, sy) = (w * 0.2, hz);
    c.fill_rect(sx - u * 0.006, sy - u * 0.05, u * 0.012, u * 0.05, Cv::solid(col(Rgb(70, 76, 72), 1.)));
    let mut b = PathBuilder::new();
    b.move_to(sx - u * 0.008, sy - u * 0.05); b.line_to(sx, sy - u * 0.1); b.line_to(sx + u * 0.008, sy - u * 0.05); b.close();
    if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(Rgb(70, 76, 72), 1.))); }
    for k in 0..26 { let x = w * (0.28 + k as f32 * 0.012); c.cloud(x, hz - u * 0.008, u * (0.012 + r.f() * 0.01), Rgb(46, 52, 40), 0.9); }
    // light falling through the clouds in bands across the fields
    for &(y, x0, x1) in &[(0.8f32, 0.05f32, 0.45f32), (0.86, 0.4, 0.95), (0.93, 0.0, 0.3)] {
        c.fill_rect(w * x0, h * y, w * (x1 - x0), h * 0.025, Cv::linear(w * x0, 0., w * x1, 0., &[(0., col(Rgb(196, 170, 90), 0.)), (0.5, col(Rgb(196, 170, 90), 0.55)), (1., col(Rgb(196, 170, 90), 0.))]));
    }
    // a canal running in from the near corner, narrowing to the horizon, catching the pale sky
    let mut b = PathBuilder::new();
    b.move_to(w * 0.57, hz + h * 0.004);
    b.quad_to(w * 0.63, h * 0.84, w * 0.72, h * 1.01);
    b.line_to(w * 1.01, h * 1.01);
    b.line_to(w * 1.01, h * 0.93);
    b.quad_to(w * 0.72, h * 0.82, w * 0.6, hz + h * 0.004);
    b.close();
    if let Some(p) = b.finish() {
        c.fill(&p, Cv::linear(0., hz, 0., h, &[(0., col(Rgb(214, 206, 178), 0.85)), (1., col(Rgb(96, 106, 104), 0.85))]));
        c.clip_path(&p);
        for k in 0..18 { let y = hz + (h - hz) * (k as f32 / 18.).powf(1.6); c.line(0., y, w, y + h * 0.002, Cv::solid(col(WHITE, 0.08 + r.f() * 0.08)), 1. + k as f32 * 0.15); }
        c.clip = None;
    }
    windmill(c, w * 0.7, hz + h * 0.01, u * 0.22, 0.35, Rgb(30, 28, 22));
    brushwork(c, (w * h / 110.) as usize, u * 0.03, u * 0.007, seed + 4);
    age(c, seed + 5);
}
//// chalk hatching: parallel strokes at `ang` across a box, each laid only where `dark` says the form is in shade,
/// broken into short runs as a hand would lay them; heavier where it is darker
#[allow(clippy::too_many_arguments)]
fn hatch(c: &mut Cv, bx: (f32, f32, f32, f32), ang: f32, gap: f32, dark: &dyn Fn(f32, f32) -> f32, ink: Rgb, r: &mut Rng) {
    let (x0, y0, x1, y1) = bx;
    let (mx, my, rad) = ((x0 + x1) / 2., (y0 + y1) / 2., ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() / 2.);
    let (dx, dy, nx, ny) = (ang.cos(), ang.sin(), -ang.sin(), ang.cos());
    let step = gap * 0.8;
    let mut o = -rad;
    while o < rad {
        let thr = 0.12 + r.f() * 0.25;
        let (mut run, mut acc, mut t) = (None::<(f32, f32)>, 0f32, -rad);
        let wob = (r.f() - 0.5) * gap * 0.5;
        while t <= rad + step {
            let (x, y) = (mx + nx * (o + wob) + dx * t, my + ny * (o + wob) + dy * t);
            let d = if t <= rad { dark(x, y) } else { 0. };
            match run {
                None if d > thr => { run = Some((x, y)); acc = d; }
                Some((sx, sy)) => {
                    let len = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
                    if d <= thr * 0.8 || len > gap * (10. + r.f() * 14.) {
                        let k = acc / (len / step + 1.);
                        c.line(sx, sy, x, y, Cv::solid(col(ink, 0.18 + 0.62 * k)), 0.8 + k * 0.7);
                        run = if d > thr { Some((x + dx * gap * 0.6, y + dy * gap * 0.6)) } else { None };
                        acc = 0.;
                    } else { acc += d; }
                }
                _ => {}
            }
            t += step;
        }
        o += gap * (0.85 + r.f() * 0.3);
    }
}
fn inside(q: &[(f32, f32)], x: f32, y: f32) -> bool {
    let n = q.len();
    let s = |i: usize| { let (a, b) = (q[i], q[(i + 1) % n]); (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0) };
    let first = s(0) >= 0.;
    (1..n).all(|i| (s(i) >= 0.) == first)
}
/// IV Studies: a sheet of red-chalk studies: a sphere and a cube in light, a cloth hung from two pins, mirror script
fn studies(c: &mut Cv, t: &Theme, seed: u64) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let mut r = Rng::new(seed);
    let chalk = t.palette.accent2.mix(Rgb(170, 60, 40), 0.55);
    c.fill_all(Cv::radial(w * 0.45, h * 0.4, w.max(h) * 0.8, &[(0., col(Rgb(236, 220, 188), 1.)), (1., col(Rgb(204, 182, 140), 1.))]));
    for _ in 0..14 { c.cloud(r.f() * w, r.f() * h, u * (0.02 + r.f() * 0.06), Rgb(150, 110, 60), 0.06 + r.f() * 0.08); }
    let ch = |a: f32| Cv::solid(col(chalk, a));
    let gap = u * 0.0075;
    // the light comes from the upper left; hatching runs down to the right, as a left hand lays it
    let (lx, ly, lz) = (-0.55f32, -0.6f32, 0.58f32);
    // construction: compass circles and a measured line
    for &(x, y, rad) in &[(0.27f32, 0.33f32, 0.21f32), (0.27, 0.33, 0.27), (0.75, 0.66, 0.2)] { if let Some(p) = Cv::arc(w * x, h * y, u * rad, u * rad, 0., TAU) { c.stroke(&p, ch(0.16), 1.); } }
    c.line(w * 0.05, h * 0.62, w * 0.5, h * 0.62, ch(0.3), 1.);
    for k in 0..=9 { let x = w * (0.05 + k as f32 * 0.05); c.line(x, h * 0.615, x, h * if k % 5 == 0 { 0.636 } else { 0.627 }, ch(0.35), 1.); }
    // the sphere, shaded as light falls on it, and its shadow cast on the table
    let (sx, sy, sr) = (w * 0.27, h * 0.33, u * 0.16);
    let shadow = move |x: f32, y: f32| { let (ex, ey) = ((x - sx - sr * 0.95) / (sr * 1.15), (y - sy - sr * 0.95) / (sr * 0.24)); let q = ex * ex + ey * ey; if q < 1. { 0.75 * (1. - q).sqrt().min(0.6) / 0.6 * (1. - (ex + 1.) * 0.2) } else { 0. } };
    hatch(c, (sx - sr * 0.4, sy + sr * 0.6, sx + sr * 2.3, sy + sr * 1.3), 0.15, gap, &shadow, chalk, &mut r);
    let ball = move |x: f32, y: f32| {
        let (nx, ny) = ((x - sx) / sr, (y - sy) / sr);
        let q = nx * nx + ny * ny;
        if q >= 0.985 { return 0.; }
        let nz = (1. - q).sqrt();
        let lam = (nx * lx + ny * ly + nz * lz).max(0.);
        let bounce = ((nx * 0.6 + ny * 0.7) - 0.55).max(0.) * 0.9;    // light thrown back up from the table
        ((1. - lam).powf(1.6) - bounce).clamp(0., 1.)
    };
    hatch(c, (sx - sr, sy - sr, sx + sr, sy + sr), 0.85, gap, &ball, chalk, &mut r);
    hatch(c, (sx - sr, sy - sr, sx + sr, sy + sr), -0.35, gap * 1.3, &|x: f32, y: f32| (ball(x, y) - 0.55) * 2.2, chalk, &mut r);
    for _ in 0..3 {
        let pts: Vec<(f32, f32)> = (0..=60).map(|k| { let a = k as f32 * TAU / 60.; let j = 1. + (r.f() - 0.5) * 0.015; (sx + a.cos() * sr * j, sy + a.sin() * sr * j) }).collect();
        brush(c, &pts, u * 0.004, chalk, 0.65, &mut r);
    }
    // the cube in two-point perspective: its top in light, the left face in half-tone, the right face dark
    let (cx0, cy0, cs) = (w * 0.72, h * 0.27, u * 0.13);
    let top = [(cx0, cy0 - cs * 0.55), (cx0 + cs * 0.9, cy0 - cs * 0.2), (cx0, cy0 + cs * 0.15), (cx0 - cs * 0.8, cy0 - cs * 0.22)];
    let low = |p: (f32, f32)| (p.0, p.1 + cs);
    let (fl, fr) = ([top[3], top[2], low(top[2]), low(top[3])], [top[2], top[1], low(top[1]), low(top[2])]);
    let bxc = (cx0 - cs, cy0 - cs * 0.6, cx0 + cs, cy0 + cs * 1.2);
    hatch(c, bxc, 0.85, gap, &|x: f32, y: f32| if inside(&fl, x, y) { 0.42 } else { 0. }, chalk, &mut r);
    hatch(c, bxc, 0.85, gap, &|x: f32, y: f32| if inside(&fr, x, y) { 0.85 } else { 0. }, chalk, &mut r);
    hatch(c, bxc, -0.35, gap * 1.3, &|x: f32, y: f32| if inside(&fr, x, y) { 0.6 } else { 0. }, chalk, &mut r);
    let floor = [low(top[3]), low(top[2]), low(top[1]), (low(top[1]).0 + cs * 0.9, low(top[1]).1 + cs * 0.15), (low(top[2]).0 + cs * 0.5, low(top[2]).1 + cs * 0.3)];
    hatch(c, (cx0 - cs, cy0, cx0 + cs * 2., cy0 + cs * 1.7), 0.15, gap, &|x: f32, y: f32| if inside(&[floor[1], floor[2], floor[3], floor[4]], x, y) { 0.5 } else { 0. }, chalk, &mut r);
    for (a, b) in [(0usize, 1usize), (1, 2), (2, 3), (3, 0)] { brush(c, &[top[a], top[b]], u * 0.004, chalk, 0.75, &mut r); }
    for k in 1..4 { brush(c, &[top[k], low(top[k])], u * 0.004, chalk, 0.75, &mut r); }
    brush(c, &[low(top[1]), low(top[2]), low(top[3])], u * 0.004, chalk, 0.75, &mut r);
    // a cloth hung from two pins: it sags between them, folds fall from each pin to a rippling hem
    let (ax, ay, dw, dh) = (w * 0.57, h * 0.47, u * 0.36, u * 0.42);
    let hem = |f: f32| (ax - dw * 0.08 + dw * 1.16 * f, ay + dh + (f * PI * 7.).sin() * dh * 0.03);
    let sag = |f: f32| (ax + dw * f, ay + 4. * f * (1. - f) * dh * 0.22);
    let mut edge = vec![];
    for k in 0..=20 { edge.push(sag(k as f32 / 20.)); }
    brush(c, &edge, u * 0.004, chalk, 0.7, &mut r);
    let mut hm = vec![];
    for k in 0..=40 { hm.push(hem(k as f32 / 40.)); }
    brush(c, &hm, u * 0.004, chalk, 0.7, &mut r);
    for k in 0..=8 {
        let f = k as f32 / 8.;
        let st = if f < 0.36 { sag(0.) } else if f > 0.64 { sag(1.) } else { sag(f) };
        let en = hem(f);
        let ctrl = (en.0 + (st.0 - en.0) * 0.15, st.1 + (en.1 - st.1) * 0.55);
        let pts: Vec<(f32, f32)> = (0..=16).map(|j| { let q = j as f32 / 16.; let m = 1. - q; (m * m * st.0 + 2. * m * q * ctrl.0 + q * q * en.0, m * m * st.1 + 2. * m * q * ctrl.1 + q * q * en.1) }).collect();
        brush(c, &pts, u * 0.0035, chalk, if k == 0 || k == 8 { 0.75 } else { 0.55 }, &mut r);
        // the shade on the turning side of each fold: short strokes laid along it, deepest halfway down
        for j in (2..15).step_by(1) {
            let (p, q) = (pts[j], pts[j + 1]);
            let (tx, ty) = (q.0 - p.0, q.1 - p.1);
            let l = (tx * tx + ty * ty).sqrt().max(0.01);
            let depth = (PI * j as f32 / 16.).sin() * u * 0.022 * (0.6 + r.f() * 0.6);
            c.line(p.0, p.1, p.0 + (ty / l) * depth + tx * 0.6, p.1 - (tx / l) * depth + ty * 0.6, ch(0.32), 0.9);
        }
    }
    for &f in &[0f32, 1.] { let (px, py) = sag(f); c.circle(px, py, u * 0.006, ch(0.8)); }
    // mirror script: a small slanting hand running right to left, its letters humps, loops and descenders
    let xh = u * 0.009;
    let mut y = h * 0.71;
    while y < h * 0.95 {
        let mut x = w * 0.47;
        while x > w * 0.06 {
            let mut pts = vec![(x, y)];
            for _ in 0..2 + (r.f() * 6.) as usize {
                let lw = xh * (0.8 + r.f() * 0.6);
                match (r.f() * 6.) as usize {
                    0 => { pts.push((x - lw * 0.4, y - xh * 2.4)); pts.push((x - lw * 0.55, y - xh * 1.6)); }
                    1 => { pts.push((x - lw * 0.5, y + xh * 1.6)); pts.push((x - lw * 0.65, y + xh * 0.4)); }
                    2 => { pts.push((x - lw * 0.6, y - xh * 0.8)); pts.push((x - lw * 0.35, y - xh * 0.95)); pts.push((x - lw * 0.45, y - xh * 0.2)); }
                    _ => { pts.push((x - lw * 0.5, y - xh * (0.7 + r.f() * 0.4))); }
                }
                x -= lw;
                pts.push((x, y));
            }
            let pts: Vec<(f32, f32)> = smooth(&pts, 4).into_iter().map(|(px, py)| (px - (y - py) * 0.25, py)).collect();
            let mut b = PathBuilder::new();
            for (k, &(px, py)) in pts.iter().enumerate() { if k == 0 { b.move_to(px, py) } else { b.line_to(px, py) } }
            if let Some(p) = b.finish() { c.stroke(&p, ch(0.55 + r.f() * 0.2), 1.); }
            x -= xh * (1.6 + r.f() * 1.5);
        }
        y += xh * 4.2;
    }
    c.noise(0.07, Rgb(120, 90, 50), seed as u32);
}
fn relic_finish(c: &mut Cv, t: &Theme, seed: u32) {
    // halation: the brights bleed a little glow, as on film; then grain, and the drawer's dark corners
    let glow = crate::paint::blurred(&c.px, 6, 5);
    let (w, h) = (c.w, c.h);
    c.blend = BlendMode::Screen;
    c.alpha = 0.18;
    c.draw_image(&glow, 0., 0., w, h, true);
    c.alpha = 1.;
    c.blend = BlendMode::SourceOver;
    c.vignette(BLACK, 0.55);
    c.noise(0.09, t.palette.surface, seed);
    c.noise(0.05, BLACK, seed ^ 0x5bd1);
}

fn relic(c: &mut Cv, t: &Theme, i: usize) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    linen(c, t, 31 + i as u64);
    match i {
        // Room I, Portraits: the plate laid upright right of centre, its catalogue card tucked under the corner
        0 => {
            film(c, t, w * 0.035, -h * 0.02, h * 1.1, u * 0.12, PI / 2. + 0.02, 12, 3);
            print(c, t, w * 0.6, h * 0.5, u * 0.64, u * 0.82, -0.025, u * 0.035, Some(2), 101, |pc| portrait(pc, 7));
            tape(c, t, w * 0.6 - u * 0.3, h * 0.5 - u * 0.4, u * 0.16, u * 0.045, -0.7, 11);
            tape(c, t, w * 0.6 + u * 0.29, h * 0.5 - u * 0.42, u * 0.16, u * 0.045, 0.62, 12);
            card(c, t, w * 0.77, h * 0.72, u * 0.36, 0.035, &["Portrait of a Man in a Ruff", "Cat. 01 · Room I · Portraits", "oil on panel, Dutch school", "Inv. RS-0101"]);
            tape(c, t, w * 0.77 + u * 0.18, h * 0.72, u * 0.12, u * 0.04, 0.08, 13);
            stamp(c, t, w * 0.77 + u * 0.31, h * 0.72 + u * 0.13, u * 0.075, -0.2, "ACC.", "0101", 14);
        }
        // Room II, Still Life: the plate wide and low, a film strip run along the top edge
        1 => {
            film(c, t, -w * 0.02, h * 0.09, w * 1.05, u * 0.11, 0.015, 31, 5);
            print(c, t, w * 0.58, h * 0.53, u * 0.95, u * 0.68, 0.02, u * 0.03, Some(0), 202, |pc| still_life(pc, 9));
            tape(c, t, w * 0.58 + u * 0.45, h * 0.53 - u * 0.32, u * 0.18, u * 0.045, 0.75, 21);
            tape(c, t, w * 0.58 - u * 0.47, h * 0.53 + u * 0.33, u * 0.16, u * 0.045, 0.7, 22);
            card(c, t, w * 0.7, h * 0.8, u * 0.4, -0.03, &["Still Life with Pewter and Lemon", "Cat. 02 · Room II · Still Life", "oil on canvas, Dutch school", "Inv. RS-0202"]);
            stamp(c, t, w * 0.7 + u * 0.35, h * 0.8 + u * 0.14, u * 0.075, 0.25, "ACC.", "0202", 24);
        }
        // Room III, Landscape: the widest plate, the sky taking most of it
        2 => {
            print(c, t, w * 0.57, h * 0.47, u * 1.12, u * 0.74, -0.015, u * 0.03, Some(3), 303, |pc| landscape(pc, 13));
            tape(c, t, w * 0.57 - u * 0.5, h * 0.47 - u * 0.36, u * 0.18, u * 0.045, -0.6, 31);
            tape(c, t, w * 0.57 + u * 0.54, h * 0.47 - u * 0.35, u * 0.18, u * 0.045, 0.55, 32);
            card(c, t, w * 0.66, h * 0.81, u * 0.38, 0.02, &["Polder with a Windmill", "Cat. 03 · Room III · Landscape", "oil on panel, Dutch school", "Inv. RS-0303"]);
            film(c, t, w * 0.98, h * 0.6, u * 0.6, u * 0.1, PI / 2. + 0.12, 50, 7);
            stamp(c, t, w * 0.66 + u * 0.33, h * 0.81 + u * 0.13, u * 0.075, 0.3, "ACC.", "0303", 34);
        }
        // Room IV, Studies: the chalk sheet itself, no margin, pinned with tape at its top corners
        3 => {
            print(c, t, w * 0.58, h * 0.5, u * 0.98, u * 0.76, 0.02, 0., Some(1), 404, |pc| studies(pc, t, 15));
            tape(c, t, w * 0.58 - u * 0.47, h * 0.5 - u * 0.37, u * 0.17, u * 0.045, -0.75, 41);
            tape(c, t, w * 0.58 + u * 0.1, h * 0.5 - u * 0.4, u * 0.17, u * 0.045, 0.05, 42);
            card(c, t, w * 0.8, h * 0.82, u * 0.36, -0.04, &["Studies of Light and Drapery", "Cat. 04 · Room IV · Studies", "red chalk on laid paper", "Inv. RS-0404"]);
            stamp(c, t, w * 0.8 + u * 0.31, h * 0.82 + u * 0.12, u * 0.075, -0.15, "ACC.", "0404", 44);
            film(c, t, w * 0.01, h * 0.42, w * 0.22, u * 0.1, -0.08, 77, 9);
        }
        // Storage: the drawer as it is, every plate stacked and taped, strips and cards between them
        _ => {
            film(c, t, w * 0.3, h * 0.96, w * 0.8, u * 0.11, -0.12, 1, 11);
            print(c, t, w * 0.3, h * 0.3, u * 0.5, u * 0.38, -0.06, u * 0.02, None, 505, |pc| studies(pc, t, 17));
            print(c, t, w * 0.56, h * 0.33, u * 0.66, u * 0.44, 0.05, u * 0.022, Some(2), 506, |pc| landscape(pc, 19));
            print(c, t, w * 0.82, h * 0.42, u * 0.36, u * 0.46, -0.07, u * 0.02, Some(0), 507, |pc| portrait(pc, 21));
            print(c, t, w * 0.62, h * 0.7, u * 0.52, u * 0.38, 0.03, u * 0.02, Some(3), 508, |pc| still_life(pc, 23));
            for (k, &(x, y, a)) in [(0.47f32, 0.13f32, 0.4f32), (0.88, 0.2, -0.5), (0.35, 0.5, 0.7), (0.79, 0.62, 0.3), (0.43, 0.62, -0.6), (0.97, 0.64, 0.8)].iter().enumerate() {
                tape(c, t, w * x, h * y, u * 0.15, u * 0.042, a, 60 + k as u64);
            }
            card(c, t, w * 0.82, h * 0.78, u * 0.34, 0.06, &["Drawer 5 — unsorted", "Room V · Storage", "four plates, one strip", "Inv. RS-0500 to 0504"]);
            stamp(c, t, w * 0.82 + u * 0.29, h * 0.78 + u * 0.13, u * 0.08, 0.18, "STORED", "0500", 70);
        }
    }
    relic_finish(c, t, 19 + i as u32);
}

/* ── Inferno: a forge at night, painted in oil darks, the sparks laid on over the varnish ─────────────────── */

/// the colour of iron at heat k: 0 cold black, then blood red, the accent, the second accent, white-hot
fn heat(t: &Theme, k: f32) -> Rgb {
    let p = t.palette;
    let ramp = [Rgb(16, 8, 5), p.accent.mix(BLACK, 0.62), p.accent, p.accent2, p.accent2.mix(WHITE, 0.82)];
    let k = k.clamp(0., 1.) * 4.;
    let i = (k as usize).min(3);
    ramp[i].mix(ramp[i + 1], k - i as f32)
}
/// light from a fire, added onto whatever it falls on
fn glow(c: &mut Cv, x: f32, y: f32, r: f32, tone: Rgb, a: f32) {
    let b = c.blend;
    c.blend = BlendMode::Plus;
    c.circle(x, y, r, Cv::radial(x, y, r, &[(0., col(tone, a)), (0.3, col(tone, a * 0.4)), (1., col(tone, 0.))]));
    c.blend = b;
}
/// the smithy's dark: an umber ground coat scumbled warmer and colder, and the reach of one fire
fn smithy(c: &mut Cv, t: &Theme, fx: f32, fy: f32, reach: f32, seed: u64) {
    let (w, h) = (c.w, c.h);
    let u = w.max(h);
    let ground = t.palette.bg.mix(BLACK, 0.3);
    c.fill_all(Cv::solid(col(ground, 1.)));
    let mut r = Rng::new(seed);
    for _ in 0..36 {
        let tone = if r.f() < 0.5 { t.palette.surface.mix(t.palette.accent, 0.08) } else { ground.mix(BLACK, 0.6) };
        c.cloud(r.f() * w, r.f() * h, u * (0.06 + r.f() * 0.18), tone, 0.3 + r.f() * 0.35);
    }
    c.fill_all(Cv::radial(fx, fy, u * reach, &[(0., col(t.palette.accent.mix(BLACK, 0.45), 0.7)), (0.3, col(t.palette.accent.mix(BLACK, 0.75), 0.4)), (1., col(BLACK, 0.))]));
}
/// the light one of the palette's two text colours: steam, smoke-light, the moon
fn pale(t: &Theme) -> Rgb { let p = t.palette; if p.ink.luma() > p.surface.luma() { p.ink } else { p.surface } }
/// an irregular lump: a stone, a coal, a flake of scale
fn lump(cx: f32, cy: f32, rx: f32, ry: f32, r: &mut Rng) -> Option<tiny_skia::Path> {
    let n = 7 + (r.f() * 4.) as usize;
    let a0 = r.f() * TAU;
    let mut b = PathBuilder::new();
    for k in 0..n {
        let a = a0 + k as f32 / n as f32 * TAU;
        let m = 0.7 + r.f() * 0.35;
        let (x, y) = (cx + a.cos() * rx * m, cy + a.sin() * ry * m);
        if k == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
    }
    b.close();
    b.finish()
}
/// embers on the draught: short trails that curl as they rise and thin as they cool; now and then one close to
/// the eye, soft and out of focus
fn embers(c: &mut Cv, t: &Theme, n: usize, bx: (f32, f32, f32, f32), drift: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let u = c.w.max(c.h);
    c.blend = BlendMode::Plus;
    for _ in 0..n {
        let (x, y) = (bx.0 + r.f() * (bx.2 - bx.0), bx.1 + r.f() * (bx.3 - bx.1));
        let hot = heat(t, 0.5 + r.f() * 0.5);
        if r.f() < 0.07 {
            let rr = u * (0.005 + r.f() * 0.012);
            c.circle(x, y, rr, Cv::radial(x, y, rr, &[(0., col(hot, 0.3)), (0.75, col(hot, 0.22)), (1., col(hot, 0.))]));
            continue;
        }
        // most are specks; a few, caught in the exposure, drew a short straight-ish streak
        let streak = r.f() < 0.45;
        let len = u * (0.004 + r.f() * 0.02) * if streak { 1. } else { 0.25 };
        let mut a = -PI / 2. + drift + (r.f() - 0.5) * 0.9;
        let turn = (r.f() - 0.5) * 0.12;
        let mut pts = vec![(x, y)];
        for _ in 0..8 { a += turn; let (px, py) = pts[pts.len() - 1]; pts.push((px + a.cos() * len / 8., py + a.sin() * len / 8.)); }
        let wid = u * (0.0007 + r.f() * 0.0012);
        let al = 0.35 + r.f() * 0.55;
        ribbon(c, &pts, |s| wid * (0.35 + 0.65 * s), Cv::solid(col(hot, al)));
        let (hx, hy) = pts[8];
        c.circle(hx, hy, wid * 4., Cv::radial(hx, hy, wid * 4., &[(0., col(hot, 0.3 * al)), (1., col(hot, 0.))]));
    }
    c.blend = BlendMode::SourceOver;
}
/// a burst of sparks from (x, y): each flies its own arc under gravity and we catch the streak of its last
/// moment, cooler the further it has gone; a few of the steel ones fork into little stars
#[allow(clippy::too_many_arguments)]
fn sparks(c: &mut Cv, t: &Theme, x: f32, y: f32, n: usize, speed: f32, a0: f32, a1: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let g = speed * 0.9;
    c.blend = BlendMode::Plus;
    for _ in 0..n {
        let a = a0 + r.f() * (a1 - a0);
        let v = speed * (0.2 + r.f().powf(0.6) * 0.9);
        let (vx, vy) = (a.cos() * v, a.sin() * v);
        let t1 = 0.12 + r.f() * 0.88;
        let t0 = (t1 - 0.05 - r.f() * 0.14).max(0.);
        let pts: Vec<(f32, f32)> = (0..=8).map(|j| { let s = t0 + (t1 - t0) * j as f32 / 8.; (x + vx * s, y + vy * s + g * s * s) }).collect();
        let k = 1. - t1 * 0.65;
        let wid = (speed * 0.0035 * (0.5 + r.f())).max(0.6);
        let hot = heat(t, 0.55 + k * 0.45);
        ribbon(c, &pts, |s| wid * (0.2 + 0.8 * s), Cv::solid(col(hot, 0.45 + 0.55 * k)));
        let (hx, hy) = pts[8];
        c.circle(hx, hy, wid * 4., Cv::radial(hx, hy, wid * 4., &[(0., col(hot, 0.3 * k)), (1., col(hot, 0.))]));
        if r.f() < 0.12 {
            for _ in 0..3 + (r.f() * 3.) as usize {
                let b = r.f() * TAU;
                let l = wid * (3. + r.f() * 6.);
                c.line(hx, hy, hx + b.cos() * l, hy + b.sin() * l, Cv::solid(col(hot, 0.6 * k)), wid * 0.45);
            }
        }
    }
    c.blend = BlendMode::SourceOver;
}
/// smoke rising from (x, y): soft bodies that widen and thin as they climb, warm underneath where the fire lights them
#[allow(clippy::too_many_arguments)]
fn plume(c: &mut Cv, t: &Theme, x: f32, y: f32, rise: f32, spread: f32, tone: Rgb, lit: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let (mut px, mut py) = (x, y);
    let mut a = -PI / 2.;
    let steps = 28;
    for k in 0..steps {
        let s = k as f32 / steps as f32;
        a = (a + (r.f() - 0.5) * 0.5).clamp(-PI * 0.8, -PI * 0.2);
        px += a.cos() * rise / steps as f32 + (r.f() - 0.5) * spread * 0.25;
        py += a.sin() * rise / steps as f32;
        let rr = spread * (0.25 + s * 0.9) * (0.7 + r.f() * 0.6);
        c.cloud(px, py, rr, tone, (0.17 - s * 0.11).max(0.03));
        c.cloud(px, py + rr * 0.35, rr * 0.6, heat(t, 0.55), 0.12 * lit * (1. - s));
    }
}
/// a bed of coals: lumps black on top, the cracks between them glowing
fn coals(c: &mut Cv, t: &Theme, x0: f32, x1: f32, y: f32, depth: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let (cx, rx) = ((x0 + x1) / 2., (x1 - x0) / 2.);
    glow(c, cx, y, rx * 1.5, heat(t, 0.6), 0.55);
    if let Some(e) = Cv::ellipse(cx, y, rx, depth, 0.) {
        c.fill(&e, Cv::radial(cx, y, rx, &[(0., col(heat(t, 0.97), 1.)), (0.6, col(heat(t, 0.72), 1.)), (1., col(heat(t, 0.42), 1.))]));
    }
    // back to front, so the near lumps sit over the far ones
    let mut lumps: Vec<(f32, f32)> = (0..((rx / depth.max(1.)) * 22.) as usize).map(|_| (cx + (r.f() - 0.5) * 2. * rx, y + (r.f() - 0.5) * 2. * depth)).collect();
    lumps.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (px, py) in lumps {
        let (dx, dy) = ((px - cx) / rx, (py - y) / depth);
        if dx * dx + dy * dy > 0.9 { continue; }
        let rr = depth * (0.3 + r.f() * 0.35);
        let k = r.f() * (1. - dx.abs() * 0.6);
        if let Some(p) = lump(px, py - rr * 0.3, rr * 1.3, rr * 0.8, &mut r) {
            c.fill(&p, Cv::linear(px, py - rr, px, py + rr * 0.6, &[(0., col(heat(t, 0.04 + k * 0.06), 1.)), (0.6, col(heat(t, 0.1 + k * 0.15), 1.)), (1., col(heat(t, 0.35 + k * 0.45), 1.))]));
        }
    }
}
/// a brick wall, its courses lit by distance from (lx, ly): the face of a furnace or a forge hood
#[allow(clippy::too_many_arguments)]
fn bricks(c: &mut Cv, t: &Theme, x0: f32, y0: f32, x1: f32, y1: f32, bh: f32, lx: f32, ly: f32, reach: f32, seed: u64) {
    let mut r = Rng::new(seed);
    let mortar = t.palette.bg.mix(BLACK, 0.55);
    c.fill_rect(x0, y0, x1 - x0, y1 - y0, Cv::solid(col(mortar, 1.)));
    let base = Rgb(52, 26, 18).mix(t.palette.accent, 0.12);
    let mut y = y0;
    let mut row = 0;
    while y < y1 {
        let bw = bh * 2.3;
        let mut x = x0 - if row % 2 == 1 { bw / 2. } else { 0. };
        while x < x1 {
            let (cx, cy) = (x + bw / 2., y + bh / 2.);
            let d = ((cx - lx).powi(2) + (cy - ly).powi(2)).sqrt();
            let lit = (1. - d / reach).max(0.).powf(2.2);
            // soot: the bricks nearest the fire's breath and a scatter of others gone black
            let soot = if r.f() < 0.18 { 0.5 } else { 0. };
            let tone = base.mix(BLACK, 0.35 + r.f() * 0.35 + soot).mix(heat(t, 0.5), lit * 0.6);
            let g = bh * 0.09;
            c.fill_rect(x.max(x0) + g, y + g, (x + bw).min(x1) - x.max(x0) - g * 2., bh - g * 2.,
                Cv::linear(0., y, 0., y + bh, &[(0., col(tone.mix(heat(t, 0.6), lit * 0.3), 1.)), (1., col(tone.mix(BLACK, 0.25), 1.))]));
            x += bw;
        }
        y += bh;
        row += 1;
    }
}
/// the anvil in profile, horn to the left: its face at y, `s` its length
fn anvil(c: &mut Cv, t: &Theme, x: f32, y: f32, s: f32) -> Option<tiny_skia::Path> {
    // from the heel round under the body and the feet, back under the horn to its point; the face and the
    // step down to the table are straight, everything else is forged round
    let under = [(0.5f32, 0.16f32), (0.43, 0.19), (0.3, 0.215), (0.21, 0.28), (0.165, 0.36), (0.17, 0.45), (0.25, 0.525), (0.37, 0.565), (0.45, 0.6),
        (0.46, 0.66), (-0.41, 0.66), (-0.4, 0.6), (-0.32, 0.565), (-0.2, 0.525), (-0.135, 0.45), (-0.13, 0.36), (-0.17, 0.28), (-0.27, 0.225),
        (-0.42, 0.18), (-0.58, 0.125), (-0.74, 0.07), (-0.86, 0.042)];
    let horn = [(-0.86f32, 0.042f32), (-0.7, 0.03), (-0.5, 0.027), (-0.36, 0.027)];
    let mut b = PathBuilder::new();
    b.move_to(x - 0.24 * s, y);
    b.line_to(x + 0.5 * s, y);
    for &(px, py) in &smooth(&under.map(|(a, q)| (x + a * s, y + q * s)), 4) { b.line_to(px, py); }
    for &(px, py) in &smooth(&horn.map(|(a, q)| (x + a * s, y + q * s)), 4)[1..] { b.line_to(px, py); }
    b.line_to(x - 0.24 * s, y + 0.027 * s);
    b.close();
    let p = b.finish()?;
    let iron = iron_of(t).mix(BLACK, 0.35);
    // a rim of firelight along every edge that faces up and toward the hearth: the outline, nudged, under the body
    c.tf = Transform::from_translate(s * 0.005, -s * 0.006);
    c.fill(&p, Cv::linear(x - 0.8 * s, 0., x + 0.5 * s, 0., &[(0., col(heat(t, 0.3), 0.6)), (1., col(heat(t, 0.5), 0.9))]));
    c.tf = Transform::identity();
    c.fill(&p, Cv::linear(0., y, 0., y + 0.66 * s, &[(0., col(iron.mix(WHITE, 0.08), 1.)), (0.05, col(iron, 1.)), (0.3, col(iron.mix(BLACK, 0.3), 1.)), (1., col(iron.mix(BLACK, 0.7), 1.))]));
    // the horn is a cone: a highlight runs along its top, the shadow under it
    let top: Vec<(f32, f32)> = smooth(&horn.map(|(a, q)| (x + a * s, y + (q + 0.025) * s)), 4);
    ribbon(c, &top, |k| s * 0.022 * (0.3 + k * 0.7), Cv::solid(col(iron.mix(WHITE, 0.14), 0.7)));
    // the waist and feet are rounded: darker at their edges
    c.clip_path(&p);
    c.fill_rect(x - 0.45 * s, y + 0.2 * s, 0.9 * s, 0.46 * s, Cv::linear(x - 0.45 * s, 0., x + 0.45 * s, 0., &[(0., col(BLACK, 0.)), (0.3, col(BLACK, 0.45)), (0.45, col(BLACK, 0.)), (0.55, col(BLACK, 0.)), (0.7, col(BLACK, 0.45)), (1., col(BLACK, 0.))]));
    c.clip = None;
    Some(p)
}

/// I Ore: the furnace mouth open on the right, a heap of ore at its foot catching the light, a shovel left in it
fn ore(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let (mx, my, mw, mh) = (w * 0.76, h * 0.6, u * 0.34, u * 0.4);
    let floor = h * 0.8;
    smithy(c, t, mx, my - mh * 0.3, 0.85, 41);
    let mut r = Rng::new(43);
    // the furnace: brick from the floor up, an arched mouth, a ring of wedge bricks round the arch
    bricks(c, t, mx - mw * 1.25, 0., w, floor, u * 0.045, mx, my - mh * 0.4, u * 0.55, 44);
    c.fill_rect(mx - mw * 1.25 - u * 0.02, 0., u * 0.03, floor, Cv::linear(mx - mw * 1.25 - u * 0.02, 0., mx - mw * 1.25 + u * 0.01, 0., &[(0., col(BLACK, 0.)), (1., col(BLACK, 0.7))]));
    let arch = |b: &mut PathBuilder, grow: f32| {
        let (hw, top) = (mw / 2. + grow, my - mh + mw / 2.);
        b.move_to(mx - hw, my + grow * 0.2);
        b.line_to(mx - hw, top);
        for k in 0..=24 { let a = PI + PI * k as f32 / 24.; b.line_to(mx + a.cos() * hw, top + a.sin() * hw); }
        b.line_to(mx + hw, my + grow * 0.2);
        b.close();
    };
    let mut b = PathBuilder::new();
    arch(&mut b, u * 0.05);
    if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(heat(t, 0.3), 1.))); }
    // voussoirs
    let top = my - mh + mw / 2.;
    for k in 0..13 {
        let a = PI + PI * (k as f32 + 0.5) / 13.;
        let (r0, r1) = (mw / 2. + u * 0.004, mw / 2. + u * 0.046);
        c.line(mx + a.cos() * r0, top + a.sin() * r0, mx + a.cos() * r1, top + a.sin() * r1, Cv::solid(col(heat(t, 0.12), 1.)), u * 0.004);
    }
    let mut b = PathBuilder::new();
    arch(&mut b, 0.);
    if let Some(p) = b.finish() {
        // the chamber: its back wall red in the dark at the top, hotter down toward the fire
        c.fill(&p, Cv::linear(0., my - mh, 0., my, &[(0., col(heat(t, 0.3), 1.)), (0.55, col(heat(t, 0.5), 1.)), (1., col(heat(t, 0.75), 1.))]));
        c.clip_path(&p);
        // flames licking up it: tongues, brighter at the root
        c.blend = BlendMode::Plus;
        for _ in 0..34 {
            let x = mx + (r.f() - 0.5) * mw * 0.95;
            let hgt = mh * (0.25 + r.f() * 0.6);
            let lean = (r.f() - 0.5) * mw * 0.25;
            leaf(c, &[(x, my), (x + lean, my - hgt * 0.5), (x + lean * 1.8, my - hgt)], mw * (0.04 + r.f() * 0.07),
                Cv::linear(0., my, 0., my - hgt, &[(0., col(heat(t, 0.95), 0.5)), (1., col(heat(t, 0.6), 0.))]));
        }
        c.blend = BlendMode::SourceOver;
        c.clip = None;
    }
    coals(c, t, mx - mw * 0.47, mx + mw * 0.47, my - u * 0.012, u * 0.022, 45);
    // the sill
    c.fill_rect(mx - mw * 0.62, my, mw * 1.24, u * 0.03, Cv::linear(0., my, 0., my + u * 0.03, &[(0., col(heat(t, 0.55), 1.)), (1., col(heat(t, 0.15), 1.))]));
    glow(c, mx, my - mh * 0.35, u * 0.9, heat(t, 0.7), 0.35);
    // the floor, and the light pooled on it
    c.fill_rect(0., floor, w, h - floor, Cv::linear(0., floor, 0., h, &[(0., col(t.palette.bg.mix(BLACK, 0.2), 1.)), (1., col(BLACK, 1.))]));
    c.fill_all(Cv::radial(mx, floor, u * 0.8, &[(0., col(heat(t, 0.45), 0.4)), (1., col(BLACK, 0.))]));
    plume(c, t, mx, my - mh, h * 0.6, u * 0.12, t.palette.muted.mix(BLACK, 0.5), 1., 46);
    // the heap: stones sorted back to front, each lit on the side that faces the fire
    let (hx, hw, hh) = (w * 0.36, w * 0.25, h * 0.3);
    let rock = t.palette.muted.mix(BLACK, 0.68);
    let mound = |x: f32| floor - hh * (1. - ((x - hx) / hw).powi(2)).max(0.);
    // the heap's body first, so no daylight shows between the stones
    let mut b = PathBuilder::new();
    b.move_to(hx - hw * 1.02, floor + u * 0.005);
    for k in 0..=40 { let x = hx - hw + 2. * hw * k as f32 / 40.; b.line_to(x, mound(x) + u * 0.012); }
    b.line_to(hx + hw * 1.02, floor + u * 0.005);
    b.close();
    if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(rock.mix(BLACK, 0.6), 1.))); }
    let mut stones: Vec<(f32, f32, f32)> = (0..420).filter_map(|_| {
        let x = hx + (r.f() * 2. - 1.) * hw;
        let top = mound(x);
        if top >= floor - u * 0.01 { return None; }
        Some((x, top + r.f().powf(1.3) * (floor - top), u * (0.014 + r.f() * 0.026)))
    }).collect();
    stones.sort_by(|a, b| a.1.total_cmp(&b.1));
    let shovel_at = stones.iter().position(|s| s.1 > floor - hh * 0.35).unwrap_or(stones.len());
    for (k, &(x, y, s)) in stones.iter().enumerate() {
        if k == shovel_at {
            // the shovel left in the heap, its handle leaning out to the left
            let (bx, by) = (hx - hw * 0.15, floor - hh * 0.5);
            let (ex, ey) = (bx - u * 0.3, by - u * 0.34);
            c.line(bx, by, ex, ey, Cv::solid(col(Rgb(44, 28, 16), 1.)), u * 0.016);
            c.line(bx + u * 0.005, by, ex + u * 0.005, ey, Cv::linear(bx, by, ex, ey, &[(0., col(Rgb(150, 96, 52).mix(heat(t, 0.6), 0.3), 0.8)), (1., col(Rgb(90, 60, 34), 0.15))]), u * 0.004);
            // the D-grip
            if let Some(p) = Cv::arc(ex - u * 0.012, ey - u * 0.016, u * 0.028, u * 0.02, 0., TAU) { c.stroke(&p, Cv::solid(col(Rgb(36, 22, 12), 1.)), u * 0.008); }
            // the blade, tipped into the stones: iron, its edge catching the furnace
            let mut b = PathBuilder::new();
            b.move_to(bx - u * 0.03, by - u * 0.005);
            b.line_to(bx + u * 0.035, by - u * 0.04);
            b.line_to(bx + u * 0.08, by + u * 0.05);
            b.quad_to(bx + u * 0.04, by + u * 0.1, bx + u * 0.005, by + u * 0.075);
            b.close();
            if let Some(p) = b.finish() { c.fill(&p, Cv::linear(bx - u * 0.03, by, bx + u * 0.08, by, &[(0., col(iron_of(t).mix(BLACK, 0.4), 1.)), (0.75, col(iron_of(t), 1.)), (1., col(heat(t, 0.3), 1.))])); }
        }
        let near = (1. - ((x - mx).abs() / (w * 0.7))).max(0.);
        let lit = heat(t, 0.18 + near * 0.32);
        if let Some(p) = lump(x, y, s, s * 0.75, &mut r) {
            c.fill(&p, Cv::linear(x - s * 0.6, y + s * 0.3, x + s * 0.9, y - s * 0.4, &[(0., col(rock.mix(BLACK, 0.5), 1.)), (0.5, col(rock, 1.)), (1., col(lit, 1.))]));
        }
        // ore: the metal in the stone glints
        if r.f() < 0.3 {
            for _ in 0..2 + (r.f() * 4.) as usize {
                let (gx, gy) = (x + r.f() * s * 0.6, y + (r.f() - 0.5) * s * 0.8);
                c.circle(gx, gy, u * 0.0016 * (1. + r.f()), Cv::solid(col(heat(t, 0.75 + r.f() * 0.25), 0.4 + near * 0.6)));
            }
        }
    }
    brushwork(c, (w * h / 120.) as usize, u * 0.03, u * 0.008, 47);
    embers(c, t, 70, (mx - mw * 1.5, my - mh * 1.4, mx + mw * 0.4, my - mh * 0.2), -0.35, 48);
}

/// II Smelt: a crucible tipped from its shank, the pour falling into a row of ingot moulds that cool along the row
fn smelt(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    smithy(c, t, w * 0.5, h * 0.7, 0.8, 51);
    let mut r = Rng::new(53);
    let iron = t.palette.muted.mix(BLACK, 0.74);
    // the mould: an iron block seen from a little above, five ingot cavities along it
    let (a, bb, d) = ((w * 0.3, h * 0.73), (w * 0.94, h * 0.68), (w * 0.34, h * 0.86));
    let at = |p: f32, q: f32| (a.0 + (bb.0 - a.0) * p + (d.0 - a.0) * q, a.1 + (bb.1 - a.1) * p + (d.1 - a.1) * q);
    let quad = |c: &mut Cv, p0: f32, p1: f32, q0: f32, q1: f32, s: tiny_skia::Shader<'static>| {
        let mut b = PathBuilder::new();
        let pts = [at(p0, q0), at(p1, q0), at(p1, q1), at(p0, q1)];
        b.move_to(pts[0].0, pts[0].1);
        for p in &pts[1..] { b.line_to(p.0, p.1); }
        b.close();
        if let Some(p) = b.finish() { c.fill(&p, s); }
    };
    // front face, then top
    let (dl, dr) = (at(0., 1.), at(1., 1.));
    let mut b = PathBuilder::new();
    b.move_to(dl.0, dl.1); b.line_to(dr.0, dr.1); b.line_to(dr.0, dr.1 + h * 0.09); b.line_to(dl.0, dl.1 + h * 0.09); b.close();
    if let Some(p) = b.finish() { c.fill(&p, Cv::linear(0., dl.1, 0., dl.1 + h * 0.09, &[(0., col(iron.mix(heat(t, 0.4), 0.25), 1.)), (1., col(BLACK, 1.))])); }
    quad(c, 0., 1., 0., 1., Cv::linear(a.0, a.1, d.0, d.1, &[(0., col(iron.mix(WHITE, 0.05), 1.)), (1., col(iron.mix(heat(t, 0.4), 0.2), 1.))]));
    let heats = [-1f32, 1., 0.78, 0.58, 0.4];
    for (k, &hk) in heats.iter().enumerate() {
        let p0 = 0.05 + k as f32 * 0.19;
        quad(c, p0, p0 + 0.15, 0.22, 0.78, Cv::solid(col(BLACK, 0.85)));
        if hk < 0. { continue; }
        // the metal sits a little below the lip; the cooler ones skin over with dark crust
        quad(c, p0 + 0.008, p0 + 0.142, 0.3, 0.76, Cv::linear(at(p0, 0.3).0, at(p0, 0.3).1, at(p0, 0.76).0, at(p0, 0.76).1,
            &[(0., col(heat(t, hk * 0.92), 1.)), (1., col(heat(t, hk), 1.))]));
        if hk < 0.9 {
            for _ in 0..(40. * (1. - hk)) as usize {
                let (x, y) = at(p0 + 0.015 + r.f() * 0.11, 0.33 + r.f() * 0.4);
                if let Some(p) = lump(x, y, u * 0.006 * (1. + r.f()), u * 0.003, &mut r) { c.fill(&p, Cv::solid(col(heat(t, hk * 0.4), 0.8))); }
            }
        }
        glow(c, at(p0 + 0.07, 0.5).0, at(p0 + 0.07, 0.5).1, u * 0.14 * hk, heat(t, hk * 0.8), 0.25 * hk);
    }
    let target = at(0.24 + 0.075, 0.52);
    // the crucible, tipped on its shank; the shank's handles run off up the left
    let (fx, fy, ch) = (w * 0.27, h * 0.3, u * 0.24);
    let tilt = 100f32;
    let rot = |lx: f32, ly: f32| { let (s, co) = tilt.to_radians().sin_cos(); (fx + lx * co - ly * s, fy + lx * s + ly * co) };
    let lip = rot(0.45 * ch, -ch);
    let (sl, sr) = (rot(-0.5 * ch, -0.52 * ch), rot(0.5 * ch, -0.52 * ch));
    c.line(sl.0, sl.1, -w * 0.05, -h * 0.05, Cv::solid(col(iron.mix(BLACK, 0.3), 1.)), u * 0.02);
    c.line(sr.0, sr.1, -w * 0.02, h * 0.02, Cv::solid(col(iron.mix(BLACK, 0.3), 1.)), u * 0.018);
    c.tf = Transform::from_rotate_at(tilt, fx, fy);
    // a fireclay pot, round-bellied, glowing through: dark on the far side, hot where the metal sits against it
    let shape = vessel(c, fx, fy, ch, &[(0., 0.24), (0.05, 0.33), (0.3, 0.42), (0.7, 0.45), (0.95, 0.43), (1., 0.41)],
        Cv::linear(fx - ch * 0.45, 0., fx + ch * 0.45, 0., &[(0., col(heat(t, 0.12), 1.)), (0.3, col(heat(t, 0.38), 1.)), (0.65, col(heat(t, 0.62), 1.)), (0.9, col(heat(t, 0.5), 1.)), (1., col(heat(t, 0.28), 1.))]));
    if let Some(p) = shape {
        c.clip_path(&p);
        // the clay's own mottle: soft, not spots
        for _ in 0..26 {
            let (x, y) = (fx + (r.f() - 0.5) * ch, fy - r.f() * ch);
            c.cloud(x, y, u * (0.02 + r.f() * 0.03), heat(t, 0.15), 0.18);
        }
        c.fill_rect(fx - ch, fy - ch * 0.2, ch * 2., ch * 0.2, Cv::linear(0., fy - ch * 0.2, 0., fy, &[(0., col(BLACK, 0.)), (1., col(BLACK, 0.5))]));
        c.clip = None;
    }
    if let Some(e) = Cv::ellipse(fx, fy - ch, 0.41 * ch, 0.055 * ch, 0.) {
        c.fill(&e, Cv::solid(col(heat(t, 0.97), 1.)));
        c.stroke(&e, Cv::solid(col(heat(t, 0.55), 1.)), u * 0.006);
    }
    c.line(fx - 0.46 * ch, fy - 0.52 * ch, fx + 0.46 * ch, fy - 0.52 * ch, Cv::solid(col(iron, 1.)), u * 0.02);
    c.line(fx - 0.46 * ch, fy - 0.52 * ch - u * 0.007, fx + 0.46 * ch, fy - 0.52 * ch - u * 0.007, Cv::solid(col(heat(t, 0.5), 0.6)), u * 0.003);
    c.tf = Transform::identity();
    // the pour: a falling arc, a halo, a body, a white core
    let pts: Vec<(f32, f32)> = (0..=30).map(|j| { let s = j as f32 / 30.; (lip.0 + (target.0 - lip.0) * s.powf(0.8), lip.1 + (target.1 - lip.1) * s * s) }).collect();
    c.blend = BlendMode::Plus;
    ribbon(c, &pts, |s| u * (0.05 - s * 0.02), Cv::solid(col(heat(t, 0.6), 0.18)));
    c.blend = BlendMode::SourceOver;
    ribbon(c, &pts, |s| u * (0.017 - s * 0.006), Cv::solid(col(heat(t, 0.86), 1.)));
    ribbon(c, &pts, |s| u * (0.007 - s * 0.003), Cv::solid(col(heat(t, 1.), 1.)));
    glow(c, target.0, target.1, u * 0.3, heat(t, 0.8), 0.45);
    glow(c, lip.0, lip.1, u * 0.2, heat(t, 0.8), 0.35);
    plume(c, t, target.0, target.1, h * 0.65, u * 0.1, t.palette.muted.mix(BLACK, 0.45), 1., 54);
    brushwork(c, (w * h / 120.) as usize, u * 0.03, u * 0.008, 55);
    sparks(c, t, target.0, target.1, 160, u * 0.32, -PI - 0.3, 0.3, 56);
    embers(c, t, 50, (w * 0.2, 0., w * 0.9, h * 0.6), 0.1, 57);
}

/// III Forge: the hammer coming down on the bar, the sparks thrown off; tools hung on the left, the hearth to the right
fn forge(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let s = u * 0.62;
    let (ax, floor) = (w * 0.46, h * 0.8);
    let ay = floor - 0.66 * s;
    let (ix, iy) = (ax - 0.06 * s, ay - 0.045 * s);
    smithy(c, t, ix, iy, 0.75, 61);
    let mut r = Rng::new(63);
    // the hearth: a brick hood up on the right, coals under it
    bricks(c, t, w * 0.74, 0., w, h * 0.46, u * 0.04, w * 0.87, h * 0.5, u * 0.5, 64);
    c.fill_rect(w * 0.72, h * 0.44, w * 0.28, u * 0.03, Cv::linear(0., h * 0.44, 0., h * 0.44 + u * 0.03, &[(0., col(heat(t, 0.35), 1.)), (1., col(heat(t, 0.1), 1.))]));
    plume(c, t, w * 0.87, h * 0.46, h * 0.5, u * 0.08, t.palette.muted.mix(BLACK, 0.5), 1., 65);
    // the firepot: a brick hearth standing on the floor, the coals heaped in its top
    bricks(c, t, w * 0.72, h * 0.53, w, floor, u * 0.04, w * 0.87, h * 0.5, u * 0.35, 62);
    c.fill_rect(w * 0.72, h * 0.53, w * 0.28, u * 0.02, Cv::solid(col(heat(t, 0.2), 1.)));
    c.fill_rect(w * 0.72 - u * 0.01, h * 0.53, u * 0.02, floor - h * 0.53, Cv::linear(w * 0.72 - u * 0.01, 0., w * 0.72 + u * 0.03, 0., &[(0., col(BLACK, 0.)), (1., col(BLACK, 0.6))]));
    coals(c, t, w * 0.73, w * 1.02, h * 0.525, u * 0.026, 66);
    // the rack on the wall: tongs and hammers, black against the dark, a rim of firelight on each
    let rack = h * 0.17;
    c.line(w * 0.03, rack, w * 0.27, rack, Cv::solid(col(BLACK, 0.9)), u * 0.01);
    for k in 0..7 {
        let x = w * (0.05 + k as f32 * 0.035);
        let len = h * (0.26 + r.f() * 0.12);
        let rim = Cv::solid(col(heat(t, 0.32), 0.55));
        if k % 2 == 0 {
            let spl = u * 0.012;
            for sg in [-1f32, 1.] {
                c.line(x, rack, x + sg * spl, rack + len, Cv::solid(col(BLACK, 0.95)), u * 0.006);
                c.line(x + u * 0.002, rack, x + sg * spl + u * 0.002, rack + len, rim.clone(), u * 0.0015);
                c.line(x + sg * spl, rack + len, x + sg * spl * 0.3, rack + len + u * 0.03, Cv::solid(col(BLACK, 0.95)), u * 0.006);
            }
        } else {
            c.line(x, rack, x, rack + len, Cv::solid(col(Rgb(30, 20, 12), 1.)), u * 0.008);
            c.line(x + u * 0.003, rack, x + u * 0.003, rack + len, rim, u * 0.0015);
            c.fill_rect(x - u * 0.018, rack + len, u * 0.036, u * 0.024, Cv::linear(x - u * 0.018, 0., x + u * 0.018, 0., &[(0., col(BLACK, 1.)), (1., col(heat(t, 0.2), 1.))]));
        }
    }
    // the floor, the stump, the anvil on it
    c.fill_rect(0., floor, w, h - floor, Cv::linear(0., floor, 0., h, &[(0., col(t.palette.bg.mix(BLACK, 0.25), 1.)), (1., col(BLACK, 1.))]));
    c.fill_all(Cv::radial(ax, floor + h * 0.05, u * 0.7, &[(0., col(heat(t, 0.45), 0.35)), (1., col(BLACK, 0.))]));
    let (sx0, sx1) = (ax - 0.34 * s, ax + 0.36 * s);
    c.fill_rect(sx0, floor - u * 0.01, sx1 - sx0, h - floor + u * 0.02, Cv::linear(sx0, 0., sx1, 0., &[(0., col(Rgb(22, 14, 9), 1.)), (0.7, col(Rgb(60, 38, 24), 1.)), (1., col(Rgb(36, 22, 14), 1.))]));
    for _ in 0..30 { let x = sx0 + r.f() * (sx1 - sx0); c.line(x, floor, x + (r.f() - 0.5) * u * 0.01, h, Cv::solid(col(BLACK, 0.2 + r.f() * 0.2)), 1. + r.f() * 1.5); }
    for _ in 0..90 {
        let (x, y) = (r.f() * w, floor + r.f() * (h - floor));
        if let Some(p) = lump(x, y, u * 0.004, u * 0.002, &mut r) { c.fill(&p, Cv::solid(col(if r.f() < 0.3 { heat(t, 0.4) } else { BLACK }, 0.5))); }
    }
    if let Some(p) = anvil(c, t, ax, ay, s) {
        c.clip_path(&p);
        glow(c, ix, iy, s * 0.4, heat(t, 0.6), 0.35);
        glow(c, ax + s * 0.7, ay - s * 0.1, s * 0.7, heat(t, 0.45), 0.15);
        c.clip = None;
    }
    c.line(ax - 0.3 * s, ay + 1., ax + 0.5 * s, ay + 1., Cv::solid(col(heat(t, 0.55), 0.7)), s * 0.006);
    // the bar: white where it is struck, cooling back toward the tongs
    let (b0, b1, bt) = (ax - 0.14 * s, ax + 0.34 * s, s * 0.045);
    c.fill_rect(b0, ay - bt, b1 - b0, bt, Cv::linear(b0, 0., b1, 0., &[(0., col(heat(t, 0.9), 1.)), (0.25, col(heat(t, 1.), 1.)), (1., col(heat(t, 0.45), 1.))]));
    for sg in [0f32, 1.] {
        c.line(b1 - s * 0.04, ay - bt * (0.2 + sg * 0.6), w * 1.03, h * (0.6 + sg * 0.04), Cv::solid(col(BLACK, 1.)), u * 0.008);
        c.line(b1 - s * 0.04, ay - bt * (0.2 + sg * 0.6) - u * 0.004, w * 1.03, h * (0.6 + sg * 0.04) - u * 0.004, Cv::solid(col(heat(t, 0.4), 0.5)), u * 0.0015);
    }
    glow(c, ix, iy, s * 0.5, heat(t, 0.85), 0.5);
    // the hammer at the moment it lands, ghosts of its swing above it; the handle runs up to a hand out of frame
    let pivot = (ix + s * 0.95, iy - s * 1.25);
    let (hw, hh) = (s * 0.17, s * 0.09);
    for g in [3f32, 2., 1., 0.] {
        c.tf = Transform::from_rotate_at(-g * 6., pivot.0, pivot.1);
        c.alpha = if g == 0. { 1. } else { 0.1 };
        c.line(ix, iy - hh * 0.5, pivot.0, pivot.1, Cv::solid(col(Rgb(46, 30, 18), 1.)), s * 0.032);
        c.line(ix - s * 0.008, iy - hh * 0.5 - s * 0.006, pivot.0 - s * 0.008, pivot.1 - s * 0.006, Cv::linear(ix, iy, ix + s * 0.4, iy - s * 0.5, &[(0., col(Rgb(170, 110, 60).mix(heat(t, 0.6), 0.4), 0.8)), (1., col(Rgb(120, 80, 46), 0.))]), s * 0.007);
        // the head: a cross-peen, its striking face hot from the bar
        c.fill_rect(ix - hw / 2., iy - hh, hw, hh, Cv::linear(0., iy - hh, 0., iy, &[(0., col(iron_of(t).mix(BLACK, 0.3), 1.)), (0.75, col(iron_of(t).mix(heat(t, 0.5), 0.3), 1.)), (1., col(heat(t, 0.75), 1.))]));
        c.fill_rect(ix - hw / 2. - s * 0.012, iy - hh * 0.7, s * 0.012, hh * 0.4, Cv::solid(col(iron_of(t).mix(BLACK, 0.4), 1.)));
        c.fill_rect(ix - hw / 2., iy - hh, hw, s * 0.008, Cv::solid(col(heat(t, 0.35), 0.6)));
    }
    c.tf = Transform::identity();
    c.alpha = 1.;
    glow(c, ix, iy, s * 0.12, heat(t, 1.), 0.6);
    brushwork(c, (w * h / 120.) as usize, u * 0.03, u * 0.008, 67);
    sparks(c, t, ix, iy, 420, s * 1.1, -PI - 0.45, 0.45, 68);
    embers(c, t, 70, (0., 0., w, h * 0.75), 0.15, 69);
}
fn iron_of(t: &Theme) -> Rgb { t.palette.muted.mix(BLACK, 0.74) }

/// IV Quench: the blade going into the trough, the water boiling round it, steam going up lit from inside;
/// a barred window gives the night's cold blue
fn quench(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    let (bx, wl) = (w * 0.52, h * 0.7);
    smithy(c, t, bx, wl - h * 0.15, 0.7, 71);
    let mut r = Rng::new(73);
    // the window: moonlight, three bars, a cold shaft across the dark
    let (wx, wy, ww, wh) = (w * 0.1, h * 0.1, u * 0.2, u * 0.28);
    c.fill_rect(wx - u * 0.015, wy - u * 0.015, ww + u * 0.03, wh + u * 0.03, Cv::solid(col(BLACK, 0.7)));
    c.fill_rect(wx, wy, ww, wh, Cv::linear(wx, wy, wx + ww, wy + wh, &[(0., col(Rgb(60, 82, 118), 1.)), (1., col(Rgb(18, 26, 44), 1.))]));
    c.circle(wx + ww * 0.7, wy + wh * 0.25, u * 0.02, Cv::solid(col(Rgb(214, 222, 230), 0.85)));
    for k in 1..4 { let x = wx + ww * k as f32 / 4.; c.line(x, wy, x, wy + wh, Cv::solid(col(BLACK, 0.95)), u * 0.008); }
    c.line(wx, wy + wh * 0.5, wx + ww, wy + wh * 0.5, Cv::solid(col(BLACK, 0.95)), u * 0.008);
    let mut b = PathBuilder::new();
    b.move_to(wx, wy); b.line_to(wx + ww, wy); b.line_to(wx + ww + w * 0.32, h * 0.95); b.line_to(wx + w * 0.12, h * 0.95); b.close();
    if let Some(p) = b.finish() {
        c.blend = BlendMode::Plus;
        c.fill(&p, Cv::linear(wx, wy, wx + w * 0.2, h * 0.9, &[(0., col(Rgb(60, 80, 110), 0.12)), (1., col(Rgb(60, 80, 110), 0.))]));
        c.blend = BlendMode::SourceOver;
    }
    // the trough: planks and iron bands, the water's skin behind its front edge
    let (tx0, tx1, back, front) = (w * 0.2, w * 0.86, wl - h * 0.035, wl + h * 0.03);
    // the back plank stands above the water: its inside face in shadow, its top edge catching the blade
    c.fill_rect(tx0, back - u * 0.05, tx1 - tx0, u * 0.05, Cv::linear(0., back - u * 0.05, 0., back, &[(0., col(Rgb(60, 38, 22).mix(heat(t, 0.5), 0.2), 1.)), (0.15, col(Rgb(40, 25, 14), 1.)), (1., col(Rgb(16, 10, 6), 1.))]));
    c.fill_rect(tx0, back - u * 0.05, tx1 - tx0, u * 0.006, Cv::linear(tx0, 0., tx1, 0., &[(0., col(heat(t, 0.25), 1.)), (0.5, col(heat(t, 0.5), 1.)), (1., col(heat(t, 0.25), 1.))]));
    c.fill_rect(tx0, back, tx1 - tx0, front - back, Cv::linear(0., back, 0., front, &[(0., col(Rgb(10, 16, 18), 1.)), (1., col(Rgb(24, 30, 30), 1.))]));
    // the blade's glow in the water, reflections broken by the boil
    c.clip_path(&PathBuilder::from_rect(tiny_skia::Rect::from_xywh(tx0, back, tx1 - tx0, front - back).unwrap_or(tiny_skia::Rect::from_xywh(0., 0., 1., 1.).unwrap())));
    glow(c, bx, wl, u * 0.4, heat(t, 0.7), 0.6);
    c.blend = BlendMode::Plus;
    for _ in 0..60 {
        let y = back + r.f() * (front - back);
        let l = u * (0.02 + r.f() * 0.1) * (1. - ((y - wl) / (front - back)).abs());
        let x = bx + (r.f() - 0.5) * u * 0.5;
        c.line(x - l, y, x + l, y, Cv::solid(col(heat(t, 0.7 + r.f() * 0.3), 0.25 * (1. - (x - bx).abs() / (u * 0.3)).max(0.))), 1. + r.f() * 2.);
    }
    for k in 0..5 {
        let rr = u * (0.03 + k as f32 * 0.035);
        if let Some(p) = Cv::arc(bx, wl, rr, rr * 0.18, 0., TAU) { c.stroke(&p, Cv::solid(col(heat(t, 0.8), 0.3 - k as f32 * 0.05)), 1.5); }
    }
    for _ in 0..40 {
        let (x, y) = (bx + (r.f() - 0.5) * u * 0.14, wl + (r.f() - 0.5) * u * 0.04);
        if let Some(p) = Cv::arc(x, y, u * 0.004 * (1. + r.f() * 2.), u * 0.003 * (1. + r.f() * 2.), 0., TAU) { c.stroke(&p, Cv::solid(col(heat(t, 0.9), 0.5)), 1.2); }
    }
    c.blend = BlendMode::SourceOver;
    c.clip = None;
    // the front: thick planks with a lit top edge, iron straps wrapped over the rim, down to the floor
    let base = h * 0.93;
    c.fill_rect(0., base, w, h - base, Cv::linear(0., base, 0., h, &[(0., col(t.palette.bg.mix(BLACK, 0.3), 1.)), (1., col(BLACK, 1.))]));
    c.fill_all(Cv::radial(bx, base, u * 0.5, &[(0., col(heat(t, 0.4), 0.25)), (1., col(BLACK, 0.))]));
    c.fill_rect(tx0, front, tx1 - tx0, base - front, Cv::linear(0., front, 0., base, &[(0., col(Rgb(66, 40, 22).mix(heat(t, 0.5), 0.2), 1.)), (0.25, col(Rgb(52, 32, 18), 1.)), (1., col(Rgb(34, 20, 11), 1.))]));
    c.fill_rect(tx0 - u * 0.05, base, tx1 - tx0 + u * 0.08, u * 0.02, Cv::linear(0., base, 0., base + u * 0.02, &[(0., col(BLACK, 0.7)), (1., col(BLACK, 0.))]));
    c.fill_rect(tx0, front, tx1 - tx0, u * 0.012, Cv::linear(tx0, 0., tx1, 0., &[(0., col(Rgb(80, 52, 30), 1.)), (0.5, col(heat(t, 0.45), 1.)), (1., col(Rgb(80, 52, 30), 1.))]));
    for k in 1..3 { let y = front + (base - front) * k as f32 / 3.; c.line(tx0, y, tx1, y, Cv::solid(col(BLACK, 0.55)), u * 0.003); }
    for _ in 0..40 { let (x, y) = (tx0 + r.f() * (tx1 - tx0), front + u * 0.02 + r.f() * (base - front - u * 0.03)); c.line(x, y, x + u * (0.03 + r.f() * 0.08), y + (r.f() - 0.5) * u * 0.004, Cv::solid(col(BLACK, 0.18)), 1. + r.f()); }
    // the trough's near end, in a little perspective
    let mut b = PathBuilder::new();
    b.move_to(tx0, front); b.line_to(tx0 - u * 0.05, back - u * 0.02); b.line_to(tx0 - u * 0.05, base - u * 0.04); b.line_to(tx0, base); b.close();
    if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(Rgb(24, 14, 8), 1.))); }
    // two iron hoops run along the front, riveted through the planks
    for &y in &[front + u * 0.035, base - u * 0.06] {
        c.fill_rect(tx0, y - u * 0.011, tx1 - tx0, u * 0.022, Cv::linear(0., y - u * 0.011, 0., y + u * 0.011, &[(0., col(iron_of(t).mix(heat(t, 0.45), 0.3), 1.)), (0.45, col(iron_of(t), 1.)), (1., col(Rgb(10, 9, 9), 1.))]));
        for k in 0..14 { let x = tx0 + (tx1 - tx0) * (k as f32 + 0.5) / 14.; c.circle(x, y, u * 0.0042, Cv::solid(col(heat(t, 0.3), 0.85))); }
    }
    // the blade, held by the tang in tongs from above right: hottest at the middle, darkening into the water
    let (top, bw) = (h * 0.2, u * 0.044);
    let lean = u * 0.07;
    let mut b = PathBuilder::new();
    b.move_to(bx - bw / 2. + lean, top); b.line_to(bx + bw * 0.42 + lean, top); b.line_to(bx + bw * 0.36, wl); b.line_to(bx - bw * 0.44, wl); b.close();
    if let Some(p) = b.finish() {
        // hot in the middle; cooler at the shoulder; going dark just above the water where the steam takes it
        c.fill(&p, Cv::linear(0., top, 0., wl, &[(0., col(heat(t, 0.5), 1.)), (0.4, col(heat(t, 0.82), 1.)), (0.75, col(heat(t, 0.62), 1.)), (0.93, col(heat(t, 0.25), 1.)), (1., col(heat(t, 0.1), 1.))]));
        c.clip_path(&p);
        // the edge runs hotter than the spine; a fuller down the middle
        c.line(bx + bw * 0.38 + lean, top, bx + bw * 0.32, wl, Cv::solid(col(heat(t, 1.), 0.7)), bw * 0.14);
        c.line(bx - bw * 0.1 + lean, top + u * 0.03, bx - bw * 0.1, wl, Cv::solid(col(heat(t, 0.2), 0.45)), bw * 0.1);
        c.line(bx - bw * 0.04 + lean, top + u * 0.03, bx - bw * 0.04, wl, Cv::solid(col(heat(t, 0.95), 0.3)), bw * 0.04);
        c.line(bx - bw * 0.46 + lean, top, bx - bw * 0.46, wl, Cv::solid(col(BLACK, 0.4)), bw * 0.1);
        c.clip = None;
    }
    c.fill_rect(bx - bw * 0.18 + lean, top - u * 0.1, bw * 0.36, u * 0.1, Cv::linear(0., top - u * 0.1, 0., top, &[(0., col(iron_of(t), 1.)), (1., col(heat(t, 0.4), 1.))]));
    for sg in [-1f32, 1.] {
        c.line(bx + lean + sg * bw * 0.2, top - u * 0.07, w * 0.72 + sg * u * 0.02, -h * 0.05, Cv::solid(col(BLACK, 1.)), u * 0.01);
        c.line(bx + lean + sg * bw * 0.2 + u * 0.004, top - u * 0.07, w * 0.72 + sg * u * 0.02 + u * 0.004, -h * 0.05, Cv::solid(col(heat(t, 0.4), 0.4)), u * 0.002);
    }
    glow(c, bx, (top + wl) / 2., u * 0.35, heat(t, 0.75), 0.35);
    brushwork(c, (w * h / 120.) as usize, u * 0.03, u * 0.008, 74);
    // steam: thick at the water, rising and spreading, lit orange from the blade and cold at its top
    // steam: a dense boil at the water, rolling up and out, lit orange from the blade and cold blue at its top
    let steam = pale(t).mix(t.palette.muted, 0.3);
    for k in 0..90 {
        let s = (k as f32 / 90.).powf(0.8);
        let side = if r.f() < 0.5 { -1. } else { 1. };
        let x = bx + side * r.f().powf(0.7) * u * (0.04 + s * 0.42) + s * u * 0.08;
        let y = wl - u * 0.015 - s * h * 0.62 + (r.f() - 0.5) * u * 0.04;
        let rr = u * (0.03 + s * 0.15) * (0.6 + r.f() * 0.7);
        let near = 1. - s;
        c.cloud(x, y, rr, steam.mix(Rgb(140, 160, 196), s * 0.5), 0.22 * near + 0.05);
        if r.f() < 0.4 { glow(c, x, y + rr * 0.3, rr * 0.9, heat(t, 0.7), 0.14 * near * near); }
    }
    // spatter thrown up from the boil
    c.blend = BlendMode::Plus;
    for _ in 0..40 {
        let (x, y) = (bx + (r.f() - 0.5) * u * 0.2, wl - r.f().powf(2.) * u * 0.12);
        c.circle(x, y, u * 0.0016 * (1. + r.f()), Cv::solid(col(pale(t).mix(heat(t, 0.8), 0.5), 0.5 + r.f() * 0.4)));
    }
    c.blend = BlendMode::SourceOver;
    embers(c, t, 30, (w * 0.3, 0., w * 0.8, h * 0.5), 0.1, 76);
}

/// V Temper: the finished blade laid on leather to cool, run through straw, bronze, purple and blue,
/// smoke and the last embers drifting over it
fn temper(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let u = w.min(h);
    smithy(c, t, -w * 0.05, h * 0.2, 0.9, 81);
    let mut r = Rng::new(83);
    // the bench: dark slate, worn; the leather on it with its folds
    let bench = h * 0.36;
    c.fill_rect(0., bench, w, h - bench, Cv::linear(0., bench, 0., h, &[(0., col(Rgb(36, 32, 30), 1.)), (1., col(Rgb(12, 10, 10), 1.))]));
    c.fill_rect(0., bench - u * 0.006, w, u * 0.012, Cv::linear(0., 0., w, 0., &[(0., col(heat(t, 0.4), 0.45)), (0.5, col(heat(t, 0.3), 0.15)), (1., col(heat(t, 0.3), 0.))]));
    for _ in 0..50 { let (x, y) = (r.f() * w, bench + r.f() * (h - bench)); c.line(x, y, x + u * (0.02 + r.f() * 0.08), y + (r.f() - 0.5) * u * 0.01, Cv::solid(col(WHITE, 0.03)), 1. + r.f()); }
    // the leather: a cut hide, its edge wandering, soft folds across it
    let (lcx, lcy, lrx, lry) = (w * 0.53, h * 0.68, w * 0.4, h * 0.2);
    let mut b = PathBuilder::new();
    let n = 48;
    for k in 0..n {
        let a = k as f32 / n as f32 * TAU;
        let m = 1. + (a * 3.).sin() * 0.04 + (a * 7.).cos() * 0.025 + (r.f() - 0.5) * 0.03;
        let (x, y) = (a.cos() * lrx * m, a.sin() * lry * m);
        let (ca, sa) = (-0.24f32).sin_cos();
        let (px, py) = (lcx + x * sa - y * ca, lcy + x * ca + y * sa);
        if k == 0 { b.move_to(px, py) } else { b.line_to(px, py) }
    }
    b.close();
    if let Some(p) = b.finish() {
        c.tf = Transform::from_translate(u * 0.008, u * 0.014);
        c.fill(&p, Cv::solid(col(BLACK, 0.4)));
        c.tf = Transform::identity();
        c.fill(&p, Cv::radial(lcx - lrx * 0.3, lcy - lry * 0.3, lrx * 1.2, &[(0., col(Rgb(50, 30, 18), 1.)), (1., col(Rgb(18, 10, 6), 1.))]));
        c.clip_path(&p);
        for k in 0..8 {
            let y = lcy - lry + lry * 2. * (0.08 + k as f32 * 0.12);
            let ctrl = [(lcx - lrx, y + lry * 0.3), (lcx, y - lry * 0.1 + (r.f() - 0.5) * lry * 0.2), (lcx + lrx, y - lry * 0.5)];
            leaf(c, &ctrl, lry * 0.12, Cv::solid(col(if k % 2 == 0 { BLACK } else { Rgb(140, 90, 52) }, 0.12)));
        }
        c.clip = None;
        c.stroke(&p, Cv::solid(col(Rgb(20, 10, 5), 0.6)), u * 0.003);
    }
    // a file left at the top of the bench
    c.tf = Transform::from_rotate_at(4., w * 0.25, h * 0.42);
    c.fill_rect(w * 0.08, h * 0.41, w * 0.3, u * 0.022, Cv::linear(0., h * 0.41, 0., h * 0.41 + u * 0.022, &[(0., col(Rgb(90, 88, 86), 1.)), (1., col(Rgb(30, 30, 30), 1.))]));
    for k in 0..90 { let x = w * 0.08 + w * 0.3 * k as f32 / 90.; c.line(x, h * 0.41, x + u * 0.006, h * 0.41 + u * 0.022, Cv::solid(col(BLACK, 0.35)), 1.); }
    c.fill_rect(w * 0.38, h * 0.41 + u * 0.006, w * 0.08, u * 0.01, Cv::solid(col(Rgb(40, 30, 20), 1.)));
    c.tf = Transform::identity();
    // the blade, in its own frame: x along it from the shoulder, y across it from spine (-) to edge (+)
    let (ox, oy) = (w * 0.22, h * 0.76);
    let ang = -24f32;
    let (len, bw) = (w * 0.66, u * 0.11);
    let outline = |b: &mut PathBuilder| {
        b.move_to(0., -bw / 2.);
        b.line_to(len * 0.8, -bw / 2.);
        b.quad_to(len * 0.94, -bw / 2., len, -bw * 0.3);
        b.quad_to(len * 0.86, bw * 0.48, len * 0.6, bw / 2.);
        b.line_to(0., bw / 2.);
        b.close();
    };
    let frame = Transform::from_rotate(ang).post_translate(ox, oy);
    // the shadow it throws on the leather
    for k in 0..4 {
        c.tf = frame.post_translate(u * (0.006 + k as f32 * 0.004), u * (0.012 + k as f32 * 0.006));
        let mut b = PathBuilder::new();
        outline(&mut b);
        if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(BLACK, 0.18))); }
    }
    c.tf = frame;
    // the tang, black with fire-scale, and its rivet holes
    c.fill_rect(-len * 0.24, -bw * 0.45, len * 0.25, bw * 0.4, Cv::linear(0., -bw * 0.45, 0., -bw * 0.05, &[(0., col(Rgb(60, 56, 54), 1.)), (1., col(Rgb(16, 14, 14), 1.))]));
    for k in 0..3 { c.circle(-len * (0.05 + k as f32 * 0.07), -bw * 0.25, bw * 0.05, Cv::solid(col(BLACK, 1.))); }
    let mut b = PathBuilder::new();
    outline(&mut b);
    if let Some(p) = b.finish() {
        // the temper colours, set across the width: bright steel at the edge, straw, bronze, purple, blue at the spine
        // oxide colours are thin films on steel: the grey shows through every one of them
        let steel = Rgb(132, 134, 136);
        let film = |c: Rgb| col(c.mix(steel, 0.38), 1.);
        c.fill(&p, Cv::linear(0., bw / 2., 0., -bw / 2., &[(0., col(Rgb(206, 206, 200), 1.)), (0.1, film(Rgb(222, 200, 130))), (0.3, film(Rgb(190, 126, 62))),
            (0.5, film(Rgb(130, 70, 112))), (0.72, film(Rgb(66, 88, 150))), (1., film(Rgb(48, 60, 88)))]));
        c.clip_path(&p);
        // along the length too: the shoulder ran hottest and holds the deepest colours
        c.fill_rect(0., -bw, len, bw * 2., Cv::linear(0., 0., len, 0., &[(0., col(Rgb(40, 30, 70), 0.45)), (0.4, col(BLACK, 0.)), (1., col(WHITE, 0.08))]));
        c.line(len * 0.03, -bw * 0.16, len * 0.62, -bw * 0.16, Cv::solid(col(BLACK, 0.35)), bw * 0.07);
        c.line(len * 0.03, -bw * 0.1, len * 0.62, -bw * 0.1, Cv::solid(col(WHITE, 0.25)), bw * 0.025);
        c.line(0., bw * 0.24, len * 0.7, bw * 0.24, Cv::solid(col(WHITE, 0.2)), bw * 0.02);
        c.blend = BlendMode::Plus;
        c.fill_rect(0., -bw * 0.46, len, bw * 0.1, Cv::linear(0., 0., len, 0., &[(0., col(heat(t, 0.6), 0.)), (0.3, col(heat(t, 0.7), 0.35)), (0.7, col(WHITE, 0.12)), (1., col(WHITE, 0.))]));
        c.blend = BlendMode::SourceOver;
        for _ in 0..60 { let (x, y) = (r.f() * len * 0.12, (r.f() - 0.5) * bw); c.circle(x, y, bw * 0.02 * (1. + r.f()), Cv::solid(col(BLACK, 0.3))); }
        c.clip = None;
    }
    // the smith's mark struck at the shoulder
    if let Some(p) = Cv::arc(len * 0.07, -bw * 0.02, bw * 0.17, bw * 0.17, 0., TAU) { c.stroke(&p, Cv::solid(col(BLACK, 0.55)), bw * 0.025); }
    let mw = crate::paint::measure("RS", "Alfa Slab One", bw * 0.17);
    c.text("RS", "Alfa Slab One", bw * 0.17, len * 0.07 - mw / 2., bw * 0.04, col(BLACK, 0.6));
    c.tf = Transform::identity();
    brushwork(c, (w * h / 140.) as usize, u * 0.03, u * 0.007, 84);
    // what is left of the fire: smoke drifting across from the left, a few embers in it
    for k in 0..3 { plume(c, t, -w * 0.02, h * (0.38 - k as f32 * 0.08), w * 0.5, u * 0.09, t.palette.muted.mix(BLACK, 0.4), 0.6, 85 + k); }
    embers(c, t, 45, (0., 0., w * 0.75, h * 0.42), 0.9, 88);
}

fn inferno_finish(c: &mut Cv, t: &Theme, seed: u32) {
    // bloom: the brights spill, as hot metal does in the eye; then the dark corners and a little grain
    let bloom = crate::paint::blurred(&c.px, 8, 6);
    let (w, h) = (c.w, c.h);
    c.blend = BlendMode::Screen;
    c.alpha = 0.4;
    c.draw_image(&bloom, 0., 0., w, h, true);
    c.alpha = 1.;
    c.blend = BlendMode::SourceOver;
    c.vignette(BLACK, 0.6);
    c.noise(0.05, t.palette.accent.mix(BLACK, 0.5), seed);
    c.noise(0.06, BLACK, seed ^ 0x9e37);
}

fn inferno(c: &mut Cv, t: &Theme, i: usize) {
    match i {
        0 => ore(c, t),
        1 => smelt(c, t),
        2 => forge(c, t),
        3 => quench(c, t),
        _ => temper(c, t),
    }
    c.reset();
    inferno_finish(c, t, 90 + i as u32);
}

/* ── Schematic: an engineer's drawing set for a battle ──────────────────────────────────────────────────── */
// Five sheets of one set: the plan of the field, a section through the walls, the engine in elevation, the blade
// in detail, and the notes with the trace of the fort. Pale pencil on slate, construction lines in blue, markup in
// warning yellow; under the tracing, a battle painting shows through in smoke, fire-lit dust and a stand of lances.

/// the drafting kit for one sheet: its scale, and the colours of line, construction and markup
struct Pen { u: f32, ink: Rgb, cons: Rgb, mark: Rgb, paper: Rgb, mono: String, disp: String }
fn sc_pen(c: &Cv, t: &Theme) -> Pen {
    let p = t.palette;
    Pen { u: c.w.max(c.h), ink: pale(t), cons: p.accent2, mark: p.accent, paper: p.bg, mono: t.mono.clone(), disp: t.display.clone() }
}
impl Pen {
    /// a line weight: 1 is a fine line
    fn w(&self, k: f32) -> f32 { (self.u * 0.00062 * k).max(0.7 * k) }
    /// a text size: 1 is a note
    fn fs(&self, k: f32) -> f32 { (self.u * 0.0074 * k).max(7.) }
    fn s(&self, tone: Rgb, a: f32) -> tiny_skia::Shader<'static> { Cv::solid(col(tone, a)) }
}

fn sc_poly(pts: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    for (j, &(x, y)) in pts.iter().enumerate() { if j == 0 { b.move_to(x, y) } else { b.line_to(x, y) } }
    if close { b.close(); }
    b.finish()
}
fn sc_rect(x: f32, y: f32, w: f32, h: f32) -> Option<tiny_skia::Path> { tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect) }
fn sc_shape(c: &mut Cv, path: &Option<tiny_skia::Path>, fill: Option<tiny_skia::Shader<'static>>, line: tiny_skia::Shader<'static>, w: f32) {
    if let Some(p) = path { if let Some(f) = fill { c.fill(p, f); } c.stroke(p, line, w); }
}
/// points round an ellipse, turned by `rot`; clockwise or not (a hole runs the other way to its outline)
fn sc_ring(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, n: usize, cw: bool) -> Vec<(f32, f32)> {
    (0..n).map(|j| {
        let a = j as f32 / n as f32 * TAU * if cw { 1. } else { -1. };
        let (x, y) = (a.cos() * rx, a.sin() * ry);
        (cx + x * rot.cos() - y * rot.sin(), cy + x * rot.sin() + y * rot.cos())
    }).collect()
}
/// one path from several closed outlines (outline one way, holes the other)
fn sc_compound(rings: &[Vec<(f32, f32)>]) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    for r in rings { for (j, &(x, y)) in r.iter().enumerate() { if j == 0 { b.move_to(x, y) } else { b.line_to(x, y) } } b.close(); }
    b.finish()
}

fn sc_text(c: &mut Cv, s: &str, fam: &str, size: f32, x: f32, y: f32, align: f32, tone: Rgb, a: f32) {
    let w = if align > 0. { crate::paint::measure(s, fam, size) } else { 0. };
    c.text(s, fam, size, x - w * align, y, col(tone, a));
}
/// text turned to an angle about (x, y), centred on it
fn sc_turned(c: &mut Cv, s: &str, fam: &str, size: f32, x: f32, y: f32, ang: f32, tone: Rgb, a: f32) {
    let w = crate::paint::measure(s, fam, size);
    let keep = c.tf;
    c.tf = Transform::from_rotate(ang.to_degrees()).post_translate(x, y);
    c.text(s, fam, size, -w / 2., size * 0.35, col(tone, a));
    c.tf = keep;
}

/// a broken line along a polyline; `pat` is on, off, on, off … lengths
fn sc_dash(c: &mut Cv, pts: &[(f32, f32)], pat: &[f32], s: &tiny_skia::Shader<'static>, w: f32) {
    let (mut k, mut left) = (0usize, pat[0]);
    for seg in pts.windows(2) {
        let (a, b) = (seg[0], seg[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let mut d = 0.;
        while d < len {
            let step = left.min(len - d);
            if k % 2 == 0 {
                let (t0, t1) = (d / len, (d + step) / len);
                c.line(a.0 + (b.0 - a.0) * t0, a.1 + (b.1 - a.1) * t0, a.0 + (b.0 - a.0) * t1, a.1 + (b.1 - a.1) * t1, s.clone(), w);
            }
            d += step;
            left -= step;
            if left <= 1e-3 { k += 1; left = pat[k % pat.len()].max(0.5); }
        }
    }
}
fn sc_head(c: &mut Cv, x: f32, y: f32, ang: f32, len: f32, s: tiny_skia::Shader<'static>) {
    let mut b = PathBuilder::new();
    b.move_to(x, y);
    for da in [-0.19f32, 0.19] { let a = ang + PI + da; b.line_to(x + a.cos() * len, y + a.sin() * len); }
    b.close();
    if let Some(p) = b.finish() { c.fill(&p, s); }
}
fn sc_arrow(c: &mut Cv, p: &Pen, pts: &[(f32, f32)], tone: Rgb, a: f32, dashed: bool) {
    let s = p.s(tone, a);
    if dashed { sc_dash(c, pts, &[p.u * 0.008, p.u * 0.005], &s, p.w(1.2)); } else if let Some(path) = sc_poly(pts, false) { c.stroke(&path, s.clone(), p.w(1.2)); }
    let n = pts.len();
    let (a0, b0) = (pts[n - 2], pts[n - 1]);
    sc_head(c, b0.0, b0.1, (b0.1 - a0.1).atan2(b0.0 - a0.0), p.u * 0.011, s);
}

/// a dimension between two points: extension lines, the dimension line `off` to one side, arrowheads, the figure upright on it
fn sc_dim(c: &mut Cv, p: &Pen, a: (f32, f32), b: (f32, f32), off: f32, txt: &str, tone: Rgb) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (uy, -ux);
    let sg = off.signum();
    let (gap, over) = (p.u * 0.003 * sg, p.u * 0.004 * sg);
    let (s, w) = (p.s(tone, 0.75), p.w(0.8));
    for q in [a, b] { c.line(q.0 + nx * gap, q.1 + ny * gap, q.0 + nx * (off + over), q.1 + ny * (off + over), s.clone(), w); }
    let (a1, b1) = ((a.0 + nx * off, a.1 + ny * off), (b.0 + nx * off, b.1 + ny * off));
    c.line(a1.0, a1.1, b1.0, b1.1, s.clone(), w);
    let ang = uy.atan2(ux);
    let hl = p.u * 0.0075;
    sc_head(c, a1.0, a1.1, ang + PI, hl, s.clone());
    sc_head(c, b1.0, b1.1, ang, hl, s);
    let up = if ang.abs() > PI / 2. + 0.01 { ang + PI } else { ang };
    let fs = p.fs(0.95);
    let (mx, my) = ((a1.0 + b1.0) / 2., (a1.1 + b1.1) / 2.);
    sc_turned(c, txt, &p.mono, fs, mx + up.sin() * fs * 0.75, my - up.cos() * fs * 0.75, up, tone, 0.95);
}

/// a numbered balloon, its leader running to the thing it names and ending in a dot
fn sc_balloon(c: &mut Cv, p: &Pen, x: f32, y: f32, tx: f32, ty: f32, n: &str, tone: Rgb) {
    let r = p.u * 0.0108;
    let (dx, dy) = (tx - x, ty - y);
    let l = (dx * dx + dy * dy).sqrt().max(1e-3);
    c.line(x + dx / l * r, y + dy / l * r, tx, ty, p.s(tone, 0.85), p.w(0.85));
    c.circle(tx, ty, p.u * 0.0019, p.s(tone, 1.));
    c.circle(x, y, r, p.s(p.paper, 0.94));
    if let Some(o) = PathBuilder::from_circle(x, y, r) { c.stroke(&o, p.s(tone, 1.), p.w(1.3)); }
    let fs = p.fs(1.05);
    sc_text(c, n, &p.mono, fs, x, y + fs * 0.36, 0.5, tone, 1.);
}

/// section lines: parallel strokes at `ang`, `gap` apart, inside a path
fn sc_hatch(c: &mut Cv, path: &tiny_skia::Path, ang: f32, gap: f32, s: tiny_skia::Shader<'static>, w: f32) {
    let bx = path.bounds();
    let (mx, my) = ((bx.left() + bx.right()) / 2., (bx.top() + bx.bottom()) / 2.);
    let rad = (bx.width().powi(2) + bx.height().powi(2)).sqrt() / 2. + 2.;
    let (dx, dy) = (ang.cos(), ang.sin());
    c.clip_path(path);
    let mut o = -rad;
    while o < rad {
        let (x, y) = (mx - dy * o, my + dx * o);
        c.line(x - dx * rad, y - dy * rad, x + dx * rad, y + dy * rad, s.clone(), w);
        o += gap;
    }
    c.clip = None;
}

/// the slate the set is drawn on, a little lighter where the lamp is
fn sc_paper(c: &mut Cv, t: &Theme) {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    let lit = p.bg.mix(p.surface, 0.7).mix(p.accent2, 0.07);
    c.fill_all(Cv::radial(w * 0.45, h * 0.42, w.max(h) * 0.78, &[(0., col(lit, 1.)), (1., col(p.bg.mix(BLACK, 0.3), 1.))]));
}
/// the printed grid over everything: fine squares, every fifth line heavier
fn sc_grid(c: &mut Cv, t: &Theme) {
    let (w, h) = (c.w, c.h);
    let step = w.max(h) / 128.;
    let lw = (w.max(h) / 1600.).max(1.);
    let tone = t.palette.accent2;
    let (mut x, mut k) = (0., 0);
    while x < w { c.fill_rect(x, 0., lw, h, Cv::solid(col(tone, if k % 5 == 0 { 0.13 } else { 0.055 }))); x += step; k += 1; }
    let (mut y, mut k) = (0., 0);
    while y < h { c.fill_rect(0., y, w, lw, Cv::solid(col(tone, if k % 5 == 0 { 0.13 } else { 0.055 }))); y += step; k += 1; }
}
/// the battle painting under the tracing: oil smoke, fire-lit dust, a little sky
fn sc_battle(c: &mut Cv, t: &Theme, cx: f32, cy: f32, rx: f32, ry: f32, heat: f32, seed: u64) {
    let p = t.palette;
    let mut r = Rng::new(seed);
    let tones = [p.muted.mix(p.bg, 0.25), p.accent.mix(Rgb(0x8a, 0x4a, 0x1c), 0.55), p.accent2.mix(p.ink, 0.35),
        p.muted.mix(p.bg, 0.25), Rgb(0x8c, 0x2e, 0x22), p.bg.mix(BLACK, 0.4)];
    for k in 0..46 {
        let a = r.f() * TAU;
        let d = r.f().sqrt();
        let (x, y) = (cx + a.cos() * rx * d, cy + a.sin() * ry * d);
        let warm = if k % 6 == 1 || k % 6 == 4 { heat } else { 1. };
        c.cloud(x, y, rx.min(ry) * (0.25 + r.f() * 0.45), tones[k % 6], (0.1 + r.f() * 0.16) * warm);
    }
}
/// a stand of pikes in the painting: the files of men a dark mass at the foot, the shafts leaning together, a few
/// fallen across, every head a leaf of steel
fn sc_lances(c: &mut Cv, t: &Theme, x0: f32, x1: f32, base: f32, len: f32, n: usize, seed: u64) {
    let mut r = Rng::new(seed);
    let p = t.palette;
    let (dark, steel) = (p.bg.mix(BLACK, 0.45), p.ink.mix(p.accent, 0.2));
    let u = c.w.max(c.h);
    let w = (u * 0.0014).max(1.);
    for _ in 0..(n / 2).max(4) { c.cloud(x0 + (x1 - x0) * r.f(), base + len * r.f() * 0.12, len * (0.12 + r.f() * 0.12), dark, 0.45); }
    let lean = -PI / 2. + 0.05;
    let flag = Rgb(0x8c, 0x2e, 0x22).mix(p.accent, 0.25);
    for k in 0..n {
        let x = x0 + (x1 - x0) * r.f();
        let a = if k % 9 == 0 { -PI / 2. + (r.f() - 0.5) * 1.1 } else { lean + (r.f() - 0.5) * 0.12 };
        let l = len * (0.8 + r.f() * 0.4);
        let y0 = base + r.f() * len * 0.1;
        let (dx, dy) = (a.cos(), a.sin());
        let (ex, ey) = (x + dx * l, y0 + dy * l);
        c.line(x, y0, ex, ey, Cv::solid(col(dark, 0.75)), w);
        let (hl, nx, ny) = (u * 0.012, -dy, dx);
        let head = sc_poly(&[(ex, ey), (ex + dx * hl * 0.35 + nx * hl * 0.11, ey + dy * hl * 0.35 + ny * hl * 0.11), (ex + dx * hl, ey + dy * hl),
            (ex + dx * hl * 0.35 - nx * hl * 0.11, ey + dy * hl * 0.35 - ny * hl * 0.11)], true);
        if let Some(h) = head { c.fill(&h, Cv::solid(col(steel, 0.5))); }
        // a pennon on every seventh, streaming off on the wind
        if k % 7 == 3 {
            let (fx, fy) = (ex - dx * hl * 0.2, ey - dy * hl * 0.2);
            let pen = sc_poly(&[(fx, fy), (fx + u * 0.018, fy + u * 0.004), (fx - dx * u * 0.01, fy - dy * u * 0.01)], true);
            if let Some(f) = pen { c.fill(&f, Cv::solid(col(flag, 0.45))); }
        }
    }
}
fn sc_underlay(c: &mut Cv, t: &Theme, smoke: (f32, f32, f32, f32, f32), pikes: (f32, f32, f32, f32, usize), seed: u64) {
    let (w, h) = (c.w, c.h);
    sc_paper(c, t);
    sc_battle(c, t, w * smoke.0, h * smoke.1, w * smoke.2, h * smoke.3, smoke.4, seed);
    sc_lances(c, t, w * pikes.0, w * pikes.1, h * pikes.2, h * pikes.3, pikes.4, seed + 1);
    let u = w.max(h);
    brushwork(c, (w * h / 220.) as usize, u * 0.03, u * 0.006, seed + 2);
    sc_grid(c, t);
}

/// the sheet: a border with its zone marks, and the title block in the corner
fn sc_sheet(c: &mut Cv, p: &Pen, sheet: &str, title: &str, scale: &str) {
    let (w, h) = (c.w, c.h);
    let (m, g) = (p.u * 0.014, p.u * 0.012);
    sc_shape(c, &sc_rect(m, m, w - m * 2., h - m * 2.), None, p.s(p.ink, 0.45), p.w(0.8));
    sc_shape(c, &sc_rect(m + g, m + g, w - (m + g) * 2., h - (m + g) * 2.), None, p.s(p.ink, 0.75), p.w(1.8));
    let nz = 8;
    let nl = ((h / w) * 8.).round().max(3.) as usize;
    let fs = p.fs(0.7);
    for k in 0..nz {
        let (x0, x1) = (m + g + (w - (m + g) * 2.) * k as f32 / nz as f32, m + g + (w - (m + g) * 2.) * (k + 1) as f32 / nz as f32);
        if k > 0 { for (y0, y1) in [(m, m + g), (h - m - g, h - m)] { c.line(x0, y0, x0, y1, p.s(p.ink, 0.45), p.w(0.8)); } }
        let n = (k + 1).to_string();
        for y in [m + g * 0.5, h - m - g * 0.5] { sc_text(c, &n, &p.mono, fs, (x0 + x1) / 2., y + fs * 0.35, 0.5, p.ink, 0.55); }
    }
    for k in 0..nl {
        let (y0, y1) = (m + g + (h - (m + g) * 2.) * k as f32 / nl as f32, m + g + (h - (m + g) * 2.) * (k + 1) as f32 / nl as f32);
        if k > 0 { for (x0, x1) in [(m, m + g), (w - m - g, w - m)] { c.line(x0, y0, x1, y0, p.s(p.ink, 0.45), p.w(0.8)); } }
        let l = ((b'A' + k as u8) as char).to_string();
        for x in [m + g * 0.5, w - m - g * 0.5] { sc_text(c, &l, &p.mono, fs, x, (y0 + y1) / 2. + fs * 0.35, 0.5, p.ink, 0.55); }
    }
    // the title block
    let (bw, bh) = (p.u * 0.21, p.u * 0.072);
    let (x, y) = (w - m - g - bw, h - m - g - bh);
    sc_shape(c, &sc_rect(x, y, bw, bh), Some(p.s(p.paper, 0.94)), p.s(p.ink, 0.85), p.w(1.6));
    let xs = x + bw * 0.68;
    let (r1, r2) = (y + bh * 0.36, y + bh * 0.68);
    let thin = p.s(p.ink, 0.5);
    c.line(xs, y, xs, y + bh, thin.clone(), p.w(1.));
    c.line(x, r1, xs, r1, thin.clone(), p.w(0.8));
    c.line(x, r2, xs, r2, thin.clone(), p.w(0.8));
    let cw = (xs - x) / 3.;
    for k in 1..3 { c.line(x + cw * k as f32, r2, x + cw * k as f32, y + bh, thin.clone(), p.w(0.8)); }
    let (lab, pad) = (p.fs(0.55), bh * 0.06);
    let small = |c: &mut Cv, s: &str, x: f32, y: f32| sc_text(c, s, &p.mono, lab, x + pad, y + lab * 1.15, 0., p.ink, 0.5);
    small(c, "PROJECT", x, y);
    sc_text(c, "THE CAMPAIGN", &p.disp, p.fs(1.5), x + pad, r1 - bh * 0.05, 0., p.ink, 0.95);
    small(c, "TITLE", x, r1);
    sc_text(c, title, &p.disp, p.fs(1.25), x + pad, r2 - bh * 0.04, 0., p.ink, 0.95);
    for (k, (l, v)) in [("SCALE", scale), ("DRAWN", "R.S."), ("REV", "C")].into_iter().enumerate() {
        let cx = x + cw * k as f32;
        small(c, l, cx, r2);
        sc_text(c, v, &p.mono, p.fs(0.9), cx + pad, y + bh - bh * 0.06, 0., p.ink, 0.9);
    }
    small(c, "SHEET", xs, y);
    sc_text(c, sheet, &p.disp, p.fs(2.7), (xs + x + bw) / 2., y + bh * 0.72, 0.5, p.mark, 1.);
    sc_text(c, "OF A-05", &p.mono, lab, (xs + x + bw) / 2., y + bh - bh * 0.08, 0.5, p.ink, 0.6);
}

/// a unit as the staff draw it: ours a box (crossed for foot, slashed for horse, a dot for guns), theirs a diamond
fn sc_unit(c: &mut Cv, p: &Pen, x: f32, y: f32, s: f32, kind: u8, foe: bool, size: &str) {
    let tone = if foe { p.mark } else { p.ink };
    let (hw, hh) = if foe { (s * 0.62, s * 0.62) } else { (s * 0.5, s * 0.32) };
    let outline = if foe { sc_poly(&[(x, y - hh), (x + hw, y), (x, y + hh), (x - hw, y)], true) } else { sc_rect(x - hw, y - hh, hw * 2., hh * 2.) };
    sc_shape(c, &outline, Some(p.s(p.paper, 0.85)), p.s(tone, 1.), p.w(1.5));
    let (ix, iy) = if foe { (s * 0.31, s * 0.31) } else { (hw, hh) };
    let ln = p.s(tone, 0.95);
    match kind {
        0 => { c.line(x - ix, y - iy, x + ix, y + iy, ln.clone(), p.w(1.1)); c.line(x - ix, y + iy, x + ix, y - iy, ln, p.w(1.1)); }
        1 => c.line(x - ix, y + iy, x + ix, y - iy, ln, p.w(1.1)),
        _ => c.circle(x, y, s * 0.09, ln),
    }
    sc_text(c, size, &p.mono, p.fs(0.7), x, y - hh - p.u * 0.003, 0.5, tone, 0.9);
}
/// the main effort: a broad hollow arrow along a sweep, narrowing as it goes, its axis dashed down the middle
fn sc_thrust(c: &mut Cv, p: &Pen, ctrl: &[(f32, f32)], wid: f32, tone: Rgb) {
    let sp = smooth(ctrl, 18);
    let n = sp.len();
    let (mut acc, mut cut) = (0., n - 1);
    while cut > 1 && acc < wid * 2. { acc += ((sp[cut].0 - sp[cut - 1].0).powi(2) + (sp[cut].1 - sp[cut - 1].1).powi(2)).sqrt(); cut -= 1; }
    let half = |j: usize| wid * 0.5 * (0.72 + 0.28 * (1. - j as f32 / cut as f32));
    let mut out: Vec<(f32, f32)> = (0..=cut).map(|j| { let (nx, ny) = normal(&sp, j); (sp[j].0 + nx * half(j), sp[j].1 + ny * half(j)) }).collect();
    let (nx, ny) = normal(&sp, cut);
    let (bx, by) = sp[cut];
    out.push((bx + nx * wid * 0.85, by + ny * wid * 0.85));
    out.push(sp[n - 1]);
    out.push((bx - nx * wid * 0.85, by - ny * wid * 0.85));
    out.extend((0..=cut).rev().map(|j| { let (nx, ny) = normal(&sp, j); (sp[j].0 - nx * half(j), sp[j].1 - ny * half(j)) }));
    sc_shape(c, &sc_poly(&out, true), Some(p.s(tone, 0.16)), p.s(tone, 1.), p.w(1.8));
    sc_dash(c, &sp[..=cut], &[p.u * 0.01, p.u * 0.006], &p.s(tone, 0.55), p.w(0.8));
}
fn sc_north(c: &mut Cv, p: &Pen, x: f32, y: f32, s: f32) {
    if let Some(o) = PathBuilder::from_circle(x, y, s) { c.stroke(&o, p.s(p.ink, 0.7), p.w(1.)); }
    let (tip, notch) = ((x, y - s * 1.3), (x, y + s * 0.45));
    sc_shape(c, &sc_poly(&[tip, (x - s * 0.36, y + s * 0.85), notch], true), Some(p.s(p.ink, 0.95)), p.s(p.ink, 1.), p.w(1.));
    sc_shape(c, &sc_poly(&[tip, (x + s * 0.36, y + s * 0.85), notch], true), Some(p.s(p.paper, 0.9)), p.s(p.ink, 1.), p.w(1.));
    sc_text(c, "N", &p.disp, p.fs(1.5), x, y - s * 1.45, 0.5, p.ink, 1.);
}
fn sc_scale(c: &mut Cv, p: &Pen, x: f32, y: f32, seg: f32, unit: &str) {
    for k in 0..4 {
        let fill = if k % 2 == 0 { p.ink } else { p.paper };
        sc_shape(c, &sc_rect(x + seg * k as f32, y, seg, seg * 0.16), Some(p.s(fill, 0.9)), p.s(p.ink, 1.), p.w(0.9));
    }
    let fs = p.fs(0.75);
    for k in 0..=4 { sc_text(c, &(k * 100).to_string(), &p.mono, fs, x + seg * k as f32, y - fs * 0.5, 0.5, p.ink, 0.85); }
    sc_text(c, unit, &p.mono, fs, x + seg * 4. + fs, y + seg * 0.15, 0., p.ink, 0.7);
}
/// a few lines of notes in the hand of the office
fn sc_notes(c: &mut Cv, p: &Pen, x: f32, y: f32, head: &str, lines: &[&str]) -> f32 {
    let hs = p.fs(1.55);
    sc_text(c, head, &p.disp, hs, x, y, 0., p.ink, 1.);
    let tw = crate::paint::measure(head, &p.disp, hs);
    c.line(x, y + hs * 0.3, x + tw, y + hs * 0.3, p.s(p.mark, 0.9), p.w(1.6));
    let fs = p.fs(0.95);
    let mut yy = y + hs * 0.6;
    for l in lines { yy += fs * 1.7; sc_text(c, l, &p.mono, fs, x, yy, 0., p.ink, 0.85); }
    yy
}

/// A-01 Plan: the field from above: the hill in contours, the river, both lines of battle and the left hook
fn plan_sheet(c: &mut Cv, t: &Theme) {
    sc_underlay(c, t, (0.72, 0.44, 0.3, 0.38, 1.), (0.6, 0.86, 0.74, 0.36, 60), 301);
    let p = sc_pen(c, t);
    let (w, h, u) = (c.w, c.h, p.u);
    let mut r = Rng::new(305);
    let (ink, cons, mark) = (p.ink, p.cons, p.mark);
    // the hill: a contour every five paces of rise; every fourth one heavier and figured
    let (hx, hy) = (w * 0.24, h * 0.42);
    let (s1, s2) = (r.f() * TAU, r.f() * TAU);
    for k in 0..9 {
        let rad = u * (0.02 + k as f32 * 0.014);
        let pts: Vec<(f32, f32)> = (0..96).map(|j| {
            let a = j as f32 / 96. * TAU;
            let q = 1. + 0.1 * (3. * a + s1 + k as f32 * 0.12).sin() + 0.07 * (5. * a + s2).sin() * (k as f32 / 8.);
            (hx + a.cos() * rad * q * 1.25, hy + a.sin() * rad * q * 0.8)
        }).collect();
        let index = k % 4 == 0;
        if let Some(path) = sc_poly(&pts, true) { c.stroke(&path, p.s(cons, if index { 0.75 } else { 0.4 }), p.w(if index { 1.4 } else { 0.8 })); }
        if index && k > 0 { let (lx, ly) = pts[70]; sc_text(c, &(124 - k * 5).to_string(), &p.mono, p.fs(0.75), lx, ly - u * 0.003, 0.5, cons, 0.9); }
    }
    // the river, its water ruled across, and the bridge
    let river = smooth(&[(-0.02 * w, 0.93 * h), (0.18 * w, 0.87 * h), (0.4 * w, 0.91 * h), (0.6 * w, 0.85 * h), (0.8 * w, 0.88 * h), (1.02 * w, 0.81 * h)], 20);
    let bank = |o: f32| -> Vec<(f32, f32)> { river.iter().enumerate().map(|(j, &(x, y))| { let (nx, ny) = normal(&river, j); (x + nx * o, y + ny * o) }).collect() };
    let (b1, b2) = (bank(h * 0.018), bank(-h * 0.018));
    let mut both = b1.clone();
    both.extend(b2.iter().rev());
    if let Some(path) = sc_poly(&both, true) { c.fill(&path, p.s(cons, 0.14)); sc_hatch(c, &path, 0., u * 0.0055, p.s(cons, 0.25), p.w(0.6)); }
    for b in [&b1, &b2] { if let Some(path) = sc_poly(b, false) { c.stroke(&path, p.s(cons, 0.85), p.w(1.2)); } }
    let j = river.iter().enumerate().min_by(|a, b| (a.1 .0 - w * 0.5).abs().total_cmp(&(b.1 .0 - w * 0.5).abs())).map(|(j, _)| j).unwrap_or(0);
    let ((bx, by), (nx, ny)) = (river[j], normal(&river, j));
    let (tx, ty) = (ny, -nx);
    for sd in [-1f32, 1.] {
        let (ox, oy) = (tx * u * 0.006 * sd, ty * u * 0.006 * sd);
        let (e0, e1) = ((bx + ox - nx * h * 0.032, by + oy - ny * h * 0.032), (bx + ox + nx * h * 0.032, by + oy + ny * h * 0.032));
        c.line(e0.0, e0.1, e1.0, e1.1, p.s(ink, 0.95), p.w(1.4));
        for (e, k) in [(e0, -1f32), (e1, 1.)] { c.line(e.0, e.1, e.0 + (nx * k + tx * sd) * u * 0.005, e.1 + (ny * k + ty * sd) * u * 0.005, p.s(ink, 0.95), p.w(1.2)); }
    }
    sc_text(c, "R. VEYNE", &p.mono, p.fs(0.85), w * 0.27, h * 0.86, 0.5, cons, 0.9);
    // phase lines and the objective
    for (x, name) in [(0.47, "PL ALPHA"), (0.565, "PL BRAVO")] {
        let pl = smooth(&[(w * x, h * 0.1), (w * (x - 0.015), h * 0.32), (w * (x + 0.012), h * 0.56), (w * (x - 0.01), h * 0.8)], 14);
        sc_dash(c, &pl, &[u * 0.012, u * 0.005, u * 0.002, u * 0.005], &p.s(cons, 0.7), p.w(1.));
        sc_text(c, name, &p.mono, p.fs(0.8), w * x + u * 0.004, h * 0.095, 0., cons, 0.95);
    }
    let obj: Vec<(f32, f32)> = (0..=72).map(|j| { let a = j as f32 / 72. * TAU; (w * 0.69 + a.cos() * w * 0.068, h * 0.42 + a.sin() * h * 0.25) }).collect();
    sc_dash(c, &obj, &[u * 0.009, u * 0.006], &p.s(mark, 0.85), p.w(1.4));
    sc_text(c, "OBJ ANVIL", &p.disp, p.fs(1.25), w * 0.69, h * 0.155, 0.5, mark, 1.);
    // both lines of battle
    let us = [(0.36, 0.3, 0u8, "III"), (0.385, 0.41, 0, "III"), (0.37, 0.52, 0, "III"), (0.24, 0.42, 2, "II"), (0.1, 0.69, 1, "II"), (0.155, 0.715, 1, "II")];
    for (x, y, k, e) in us { sc_unit(c, &p, w * x, h * y, u * 0.026, k, false, e); }
    for (x, y, e) in [(0.66, 0.26, "III"), (0.69, 0.37, "III"), (0.675, 0.48, "III"), (0.72, 0.6, "II"), (0.83, 0.42, "X")] {
        sc_unit(c, &p, w * x, h * y, u * 0.026, 0, true, e);
    }
    for y in [0.3, 0.41, 0.52] { sc_arrow(c, &p, &[(w * 0.405, h * y), (w * 0.455, h * (y + 0.004)), (w * 0.53, h * y)], ink, 0.75, false); }
    sc_arrow(c, &p, &[(w * 0.815, h * 0.46), (w * 0.79, h * 0.52), (w * 0.745, h * 0.57)], mark, 0.85, true);
    sc_thrust(c, &p, &[(w * 0.13, h * 0.75), (w * 0.29, h * 0.79), (w * 0.45, h * 0.76), (w * 0.56, h * 0.67), (w * 0.635, h * 0.585)], u * 0.024, mark);
    // the orders, and what they point at
    sc_notes(c, &p, w * 0.055, h * 0.1, "ORDER OF BATTLE", &["1  GUNS ON HILL 124", "2  HORSE HELD FOR THE HOOK", "3  MAIN EFFORT: LEFT HOOK", "4  OBJECTIVE ANVIL"]);
    sc_balloon(c, &p, w * 0.14, h * 0.33, w * 0.24, h * 0.42, "1", ink);
    sc_balloon(c, &p, w * 0.06, h * 0.6, w * 0.1, h * 0.69, "2", ink);
    sc_balloon(c, &p, w * 0.4, h * 0.88, w * 0.44, h * 0.77, "3", mark);
    sc_balloon(c, &p, w * 0.6, h * 0.12, w * 0.645, h * 0.21, "4", mark);
    sc_north(c, &p, w * 0.915, h * 0.19, u * 0.022);
    sc_scale(c, &p, w * 0.79, h * 0.31, u * 0.022, "PACES");
    sc_sheet(c, &p, "A-01", "PLAN OF THE FIELD", "1 : 2500");
}

/// A-02 Section: a cut through the walls: glacis, ditch, scarp, rampart and parapet, and the mine dug under them
fn section_sheet(c: &mut Cv, t: &Theme) {
    sc_underlay(c, t, (0.6, 0.2, 0.34, 0.2, 1.3), (0.06, 0.27, 0.58, 0.3, 70), 311);
    let p = sc_pen(c, t);
    let (w, h, u) = (c.w, c.h, p.u);
    let (ink, cons, mark) = (p.ink, p.cons, p.mark);
    let at = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> { pts.iter().map(|&(x, y)| (w * x, h * y)).collect() };
    // the cut earth, from the field over the glacis, down into the ditch, up the rampart and down into the town
    let surface = [(0.03, 0.62), (0.3, 0.54), (0.33, 0.54), (0.33, 0.8), (0.5, 0.8), (0.52, 0.36), (0.548, 0.36), (0.566, 0.3), (0.61, 0.3),
        (0.61, 0.335), (0.625, 0.335), (0.625, 0.36), (0.72, 0.36), (0.83, 0.62), (0.97, 0.62)];
    let mut earth = at(&surface);
    // the cut is broken off at the foot with a freehand break line
    for j in 0..=40 { let f = 1. - j as f32 / 40.; earth.push((w * (0.03 + 0.94 * f), h * (0.93 + 0.006 * (f * 37.).sin()))); }
    let earth_p = sc_poly(&earth, true);
    if let Some(e) = &earth_p { c.fill(e, p.s(p.paper, 0.6)); sc_hatch(c, e, -PI / 4., u * 0.011, p.s(ink, 0.25), p.w(0.7)); c.stroke(e, p.s(ink, 0.45), p.w(0.9)); }
    // water standing in the ditch
    let wl = 0.72;
    let water = sc_poly(&at(&[(0.33, wl), (0.5 + 0.02 * (0.8 - wl) / 0.44, wl), (0.5, 0.8), (0.33, 0.8)]), true);
    if let Some(wp) = &water { c.fill(wp, p.s(p.paper, 1.)); c.fill(wp, p.s(cons, 0.18)); sc_hatch(c, wp, 0., u * 0.006, p.s(cons, 0.35), p.w(0.6)); }
    let (vx, vy) = (w * 0.42, h * wl);
    sc_shape(c, &sc_poly(&[(vx, vy), (vx - u * 0.006, vy - u * 0.009), (vx + u * 0.006, vy - u * 0.009)], true), None, p.s(cons, 1.), p.w(1.));
    for k in 1..3 { let l = u * 0.009 / k as f32; c.line(vx - l, vy + u * 0.004 * k as f32, vx + l, vy + u * 0.004 * k as f32, p.s(cons, 0.9), p.w(0.9)); }
    // masonry: the counterscarp wall, and the scarp revetment on its footing, cross-hatched
    for wall in [vec![(0.315, 0.54), (0.33, 0.54), (0.33, 0.82), (0.31, 0.82)], vec![(0.5, 0.8), (0.52, 0.36), (0.548, 0.36), (0.538, 0.8), (0.545, 0.845), (0.494, 0.845)]] {
        let wp = sc_poly(&at(&wall), true);
        if let Some(q) = &wp { c.fill(q, p.s(p.paper, 1.)); sc_hatch(c, q, PI / 4., u * 0.0045, p.s(ink, 0.5), p.w(0.7)); sc_hatch(c, q, -PI / 4., u * 0.0045, p.s(ink, 0.3), p.w(0.6)); }
        sc_shape(c, &wp, None, p.s(ink, 1.), p.w(1.6));
    }
    sc_shape(c, &sc_poly(&at(&surface), false), None, p.s(ink, 1.), p.w(2.2));
    // the mine: a shaft sunk on the glacis, the gallery driven under the ditch, the charge under the scarp
    let gy = (0.86, 0.89);
    let gallery = sc_poly(&at(&[(0.135, 0.585), (0.15, 0.583), (0.15, gy.0), (0.525, gy.0), (0.525, gy.1), (0.135, gy.1)]), true);
    sc_shape(c, &gallery, Some(p.s(p.paper, 1.)), p.s(mark, 1.), p.w(1.4));
    for k in 0..12 { let x = w * (0.17 + k as f32 * 0.03); c.line(x, h * gy.0, x, h * gy.1, p.s(mark, 0.45), p.w(0.8)); }
    let (chx, chy, chr) = (w * 0.545, h * 0.875, h * 0.026);
    c.circle(chx, chy, chr, p.s(mark, 0.3));
    if let Some(o) = PathBuilder::from_circle(chx, chy, chr) { c.stroke(&o, p.s(mark, 1.), p.w(1.6)); }
    let blast: Vec<(f32, f32)> = (0..=60).map(|j| { let a = j as f32 / 60. * TAU; (chx + a.cos() * chr * 2.8, chy + a.sin() * chr * 2.8) }).collect();
    sc_dash(c, &blast, &[u * 0.006, u * 0.005], &p.s(mark, 0.6), p.w(1.));
    sc_text(c, "GALLERY", &p.mono, p.fs(0.8), w * 0.24, h * gy.0 - u * 0.004, 0.5, mark, 0.95);
    sc_text(c, "CHARGE", &p.mono, p.fs(0.8), chx + chr * 3.2, chy + u * 0.003, 0., mark, 0.95);
    // a gun on the rampart, and its line of fire grazing the glacis: why the glacis has its slope
    let (wx, wy, wr) = (w * 0.652, h * 0.36 - h * 0.017, h * 0.017);
    let barrel = sc_poly(&[(w * 0.67, h * 0.316), (w * 0.578, h * 0.297), (w * 0.578, h * 0.308), (w * 0.67, h * 0.333)], true);
    sc_shape(c, &sc_rect(w * 0.635, h * 0.33, w * 0.04, h * 0.012), Some(p.s(p.paper, 1.)), p.s(ink, 1.), p.w(1.2));
    sc_shape(c, &barrel, Some(p.s(p.paper, 1.)), p.s(ink, 1.), p.w(1.4));
    c.circle(wx, wy, wr, p.s(p.paper, 1.));
    if let Some(o) = PathBuilder::from_circle(wx, wy, wr) { c.stroke(&o, p.s(ink, 1.), p.w(1.2)); }
    sc_dash(c, &[(w * 0.575, h * 0.302), (w * 0.03, h * 0.6)], &[u * 0.014, u * 0.005, u * 0.002, u * 0.005], &p.s(mark, 0.85), p.w(1.1));
    let fa = (h * 0.6 - h * 0.302).atan2(w * 0.03 - w * 0.575) + PI;
    sc_turned(c, "LINE OF FIRE", &p.mono, p.fs(0.8), w * 0.2, h * 0.49 - u * 0.012, fa, mark, 0.95);
    sc_turned(c, "GLACIS  1 : 6", &p.mono, p.fs(0.8), w * 0.2, h * 0.565 + u * 0.03, (h * -0.08).atan2(w * 0.27), ink, 0.85);
    // dimensions
    sc_dim(c, &p, (w * 0.5, h * 0.8), (w * 0.5, h * 0.36), u * 0.03, "9 600", ink);
    sc_dim(c, &p, (w * 0.33, h * 0.54), (w * 0.5, h * 0.54), u * 0.05, "12 000", ink);
    sc_dim(c, &p, (w * 0.625, h * 0.36), (w * 0.72, h * 0.36), u * 0.06, "8 400", ink);
    sc_dim(c, &p, (w * 0.135, h * gy.0), (w * 0.135, h * 0.587), u * 0.025, "7 200", mark);
    // what is what
    let title_y = sc_notes(c, &p, w * 0.055, h * 0.11, "SECTION A–A", &["THROUGH THE NORTH CURTAIN", "ALL FIGURES IN MILLIMETRES"]);
    let _ = title_y;
    for (n, bx, by, tx, ty, tone) in [("1", 0.17, 0.44, 0.2, 0.565, ink), ("2", 0.27, 0.71, 0.322, 0.66, ink), ("3", 0.39, 0.64, 0.42, 0.76, cons),
        ("4", 0.455, 0.47, 0.524, 0.5, ink), ("5", 0.6, 0.17, 0.59, 0.3, ink), ("6", 0.79, 0.3, 0.69, 0.36, ink), ("7", 0.27, 0.8, 0.3, 0.875, mark), ("8", 0.64, 0.79, chx / w + 0.02, 0.87, mark)] {
        sc_balloon(c, &p, w * bx, h * by, w * tx, h * ty, n, tone);
    }
    sc_sheet(c, &p, "A-02", "SECTION THROUGH THE WALLS", "1 : 200");
}

/// A-03 Profile: the engine from the side, cocked; its arm's swing in ghosts, the throw and where it lands
fn elevation_sheet(c: &mut Cv, t: &Theme) {
    sc_underlay(c, t, (0.74, 0.48, 0.24, 0.4, 1.5), (0.58, 0.92, 0.8, 0.3, 64), 321);
    let p = sc_pen(c, t);
    let (w, h, u) = (c.w, c.h, p.u);
    let (ink, cons, mark) = (p.ink, p.cons, p.mark);
    let g = h * 0.8;
    // the ground, and its hatching
    c.line(w * 0.05, g, w * 0.96, g, p.s(ink, 1.), p.w(1.6));
    let mut x = w * 0.05;
    while x < w * 0.96 { c.line(x, g, x - u * 0.008, g + u * 0.008, p.s(ink, 0.4), p.w(0.7)); x += u * 0.007; }
    // the frame: sole, two legs to the axle, a cross-beam and braces
    let (px, py) = (w * 0.3, h * 0.45);
    let beam = |c: &mut Cv, a: (f32, f32), b: (f32, f32), th: f32| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(1e-3);
        let (nx, ny) = (-dy / l * th / 2., dx / l * th / 2.);
        sc_shape(c, &sc_poly(&[(a.0 + nx, a.1 + ny), (b.0 + nx, b.1 + ny), (b.0 - nx, b.1 - ny), (a.0 - nx, a.1 - ny)], true), Some(p.s(p.paper, 0.95)), p.s(ink, 1.), p.w(1.3));
    };
    beam(c, (w * 0.16, g - h * 0.014), (w * 0.44, g - h * 0.014), h * 0.028);
    beam(c, (w * 0.2, g - h * 0.028), (px, py), h * 0.022);
    beam(c, (w * 0.4, g - h * 0.028), (px, py), h * 0.022);
    beam(c, (w * 0.235, h * 0.63), (w * 0.365, h * 0.63), h * 0.016);
    beam(c, (w * 0.25, g - h * 0.028), (w * 0.29, h * 0.63), h * 0.012);
    beam(c, (w * 0.35, g - h * 0.028), (w * 0.31, h * 0.63), h * 0.012);
    // the arm's swing: the tip's circle, the arm in passing, and the cocked arm itself
    let la = h * 0.4;
    let dir = |deg: f32| { let a = deg.to_radians(); (a.cos(), a.sin()) };
    let swing: Vec<(f32, f32)> = (0..=80).map(|j| { let (dx, dy) = dir(145. + 160. * j as f32 / 80.); (px + dx * la, py + dy * la) }).collect();
    sc_dash(c, &swing, &[u * 0.01, u * 0.006], &p.s(cons, 0.7), p.w(1.));
    let (ex, ey) = swing[80];
    let (tx2, ty2) = swing[79];
    sc_head(c, ex, ey, (ey - ty2).atan2(ex - tx2), u * 0.011, p.s(cons, 0.8));
    let ratio = 3.5;
    for deg in [200f32, 250., 305.] {
        let (dx, dy) = dir(deg);
        let (a, b) = ((px - dx * la / ratio, py - dy * la / ratio), (px + dx * la, py + dy * la));
        sc_dash(c, &[a, b], &[u * 0.007, u * 0.005], &p.s(cons, 0.55), p.w(1.));
        if let Some(o) = PathBuilder::from_circle(b.0, b.1, u * 0.004) { c.stroke(&o, p.s(cons, 0.7), p.w(1.)); }
    }
    let (dx, dy) = dir(145.);
    let (tip, short) = ((px + dx * la, py + dy * la), (px - dx * la / ratio, py - dy * la / ratio));
    beam(c, short, tip, h * 0.018);
    // the counterweight hangs from the short end; the sling lies in its trough with the stone
    let (cwx, cwy) = (short.0, short.1 + h * 0.03);
    c.line(short.0, short.1, cwx, cwy, p.s(ink, 1.), p.w(1.4));
    let bx = sc_rect(cwx - h * 0.035, cwy, h * 0.07, h * 0.085);
    if let Some(b) = &bx { c.fill(b, p.s(p.paper, 1.)); sc_hatch(c, b, PI / 4., u * 0.005, p.s(ink, 0.45), p.w(0.7)); }
    sc_shape(c, &bx, None, p.s(ink, 1.), p.w(1.6));
    let stone = (w * 0.255, g - h * 0.046);
    let sling = smooth(&[tip, (w * 0.14, g - h * 0.05), (w * 0.2, g - h * 0.035), stone], 10);
    if let Some(s) = sc_poly(&sling, false) { c.stroke(&s, p.s(ink, 0.9), p.w(1.)); }
    c.circle(stone.0, stone.1, h * 0.014, p.s(p.paper, 1.));
    if let Some(o) = PathBuilder::from_circle(stone.0, stone.1, h * 0.014) { c.stroke(&o, p.s(ink, 1.), p.w(1.3)); sc_hatch(c, &o, PI / 4., u * 0.003, p.s(ink, 0.5), p.w(0.6)); }
    c.circle(px, py, h * 0.012, p.s(p.paper, 1.));
    if let Some(o) = PathBuilder::from_circle(px, py, h * 0.012) { c.stroke(&o, p.s(ink, 1.), p.w(1.5)); }
    c.circle(px, py, h * 0.004, p.s(ink, 1.));
    // the throw: from the sling's release over the apex, down onto the tower
    let (rx, ry) = { let (dx, dy) = dir(305.); (px + dx * la, py + dy * la) };
    let (tx, ty) = (w * 0.645, h * 0.56);
    let ay = h * 0.07;
    let q = ((ry - ay) / (ty - ay)).sqrt();
    let ax = (rx + q * tx) / (1. + q);
    let k = (ty - ay) / (tx - ax).powi(2);
    let arc: Vec<(f32, f32)> = (0..=60).map(|j| { let x = rx + (tx - rx) * j as f32 / 60.; (x, ay + k * (x - ax).powi(2)) }).collect();
    sc_dash(c, &arc, &[u * 0.012, u * 0.006], &p.s(mark, 0.9), p.w(1.4));
    let (lx, ly) = arc[59];
    sc_head(c, tx, ty, (ty - ly).atan2(tx - lx), u * 0.012, p.s(mark, 1.));
    if let Some(o) = PathBuilder::from_circle(ax, ay, u * 0.006) { c.stroke(&o, p.s(mark, 1.), p.w(1.2)); }
    c.line(ax - u * 0.01, ay, ax + u * 0.01, ay, p.s(mark, 1.), p.w(1.));
    c.line(ax, ay - u * 0.01, ax, ay + u * 0.01, p.s(mark, 1.), p.w(1.));
    sc_dim(c, &p, (ax, g), (ax, ay), -u * 0.012, "APEX 64 m", mark);
    // the tower it is thrown at, in the enemy's colour
    let (t0, t1, top) = (w * 0.62, w * 0.72, h * 0.45);
    let mut tw = vec![(t0, g), (t0, top)];
    for m in 0..5 { let x0 = t0 + (t1 - t0) * m as f32 / 5.; let x1 = x0 + (t1 - t0) / 10.; tw.extend([(x0, top - h * 0.03), (x1, top - h * 0.03), (x1, top), (x1 + (t1 - t0) / 10., top)]); }
    tw.extend([(t1, top), (t1, g)]);
    sc_shape(c, &sc_poly(&tw, true), Some(p.s(p.paper, 0.6)), p.s(mark, 1.), p.w(1.6));
    for (x, y) in [(0.655, 0.55), (0.685, 0.66)] { sc_shape(c, &sc_rect(w * x, h * y, w * 0.008, h * 0.05), None, p.s(mark, 0.9), p.w(1.)); }
    for k in 0..8 { let a = k as f32 / 8. * TAU + 0.3; c.line(tx + a.cos() * u * 0.006, ty + a.sin() * u * 0.006, tx + a.cos() * u * 0.018, ty + a.sin() * u * 0.018, p.s(mark, 0.9), p.w(1.1)); }
    // measures
    sc_dim(c, &p, (px, g), (px, py), -u * 0.11, "H 11 200", ink);
    sc_dim(c, &p, (px, g), (w * 0.67, g), -u * 0.035, "RANGE 290 m", ink);
    sc_text(c, "ARM 3.5 : 1", &p.mono, p.fs(0.9), w * 0.14, h * 0.5, 0.5, cons, 0.95);
    sc_notes(c, &p, w * 0.055, h * 0.11, "PROFILE", &["COUNTERPOISE ENGINE, SIDE", "SHOWN COCKED; SWING IN GHOST"]);
    for (n, bx, by, ttx, tty) in [("1", 0.37, 0.36, px / w + 0.006, py / h), ("2", short.0 / w + 0.07, 0.24, short.0 / w + 0.01, cwy / h + 0.05),
        ("3", 0.13, 0.6, 0.2, 0.575), ("4", 0.12, 0.88, 0.17, 0.75), ("5", 0.3, 0.88, stone.0 / w + 0.006, stone.1 / h), ("6", 0.47, 0.7, 0.4, 0.74)] {
        sc_balloon(c, &p, w * bx, h * by, w * ttx, h * tty, n, ink);
    }
    sc_sheet(c, &p, "A-03", "PROFILE OF THE ENGINE", "1 : 100");
}

/// A-04 Detail: the sword taken down along its axis, every part numbered; the guard face-on, the blade cut through
fn detail_sheet(c: &mut Cv, t: &Theme) {
    sc_underlay(c, t, (0.24, 0.84, 0.26, 0.18, 0.9), (0.08, 0.3, 0.97, 0.2, 40), 331);
    let p = sc_pen(c, t);
    let (w, h, u) = (c.w, c.h, p.u);
    let (ink, cons, mark) = (p.ink, p.cons, p.mark);
    let y0 = h * 0.55;
    sc_dash(c, &[(w * 0.04, y0), (w * 0.965, y0)], &[u * 0.022, u * 0.005, u * 0.003, u * 0.005], &p.s(ink, 0.4), p.w(0.8));
    let fillp = Some(p.s(p.paper, 0.92));
    let body = |c: &mut Cv, x0: f32, x1: f32, h0: f32, h1: f32| sc_shape(c, &sc_poly(&[(w * x0, y0 - h * h0), (w * x1, y0 - h * h1), (w * x1, y0 + h * h1), (w * x0, y0 + h * h0)], true), Some(p.s(p.paper, 0.92)), p.s(ink, 1.), p.w(1.5));
    // 1 the cap, 2 the grip in its diamond wrap, 3 the collar, 4 + 5 + 4 washers either side of the guard, 6 the blade collar
    let mut cap: Vec<(f32, f32)> = (0..=16).map(|j| { let a = PI / 2. + PI * j as f32 / 16.; (w * 0.08 + a.cos() * w * 0.014, y0 + a.sin() * h * 0.028) }).collect();
    cap.extend([(w * 0.088, y0 - h * 0.028), (w * 0.088, y0 + h * 0.028)]);
    sc_shape(c, &sc_poly(&cap, true), fillp.clone(), p.s(ink, 1.), p.w(1.5));
    let grip = sc_poly(&[(w * 0.1, y0 - h * 0.026), (w * 0.3, y0 - h * 0.03), (w * 0.3, y0 + h * 0.03), (w * 0.1, y0 + h * 0.026)], true);
    sc_shape(c, &grip, fillp.clone(), p.s(ink, 1.), p.w(1.5));
    if let Some(gp) = &grip { sc_hatch(c, gp, 1.05, u * 0.011, p.s(ink, 0.55), p.w(0.9)); sc_hatch(c, gp, -1.05, u * 0.011, p.s(ink, 0.55), p.w(0.9)); }
    sc_shape(c, &Cv::ellipse(w * 0.2, y0 - h * 0.03, w * 0.016, h * 0.008, 0.), fillp.clone(), p.s(ink, 1.), p.w(1.1));
    body(c, 0.31, 0.325, 0.032, 0.033);
    body(c, 0.335, 0.339, 0.05, 0.05);
    let guard = sc_rect(w * 0.349, y0 - h * 0.1, w * 0.01, h * 0.2);
    sc_shape(c, &guard, fillp.clone(), p.s(ink, 1.), p.w(1.5));
    if let Some(gp) = &guard { sc_hatch(c, gp, PI / 4., u * 0.004, p.s(ink, 0.5), p.w(0.7)); }
    body(c, 0.369, 0.373, 0.05, 0.05);
    body(c, 0.385, 0.415, 0.035, 0.027);
    // 8 the blade with its tang: the peg hole and file marks, the ridge line, the temper line, the point
    let tang = sc_poly(&[(w * 0.432, y0 - h * 0.012), (w * 0.54, y0 - h * 0.019), (w * 0.54, y0 + h * 0.021), (w * 0.44, y0 + h * 0.014), (w * 0.43, y0)], true);
    sc_shape(c, &tang, fillp.clone(), p.s(ink, 1.), p.w(1.5));
    for k in 0..9 { let x = w * (0.455 + k as f32 * 0.009); c.line(x, y0 - h * 0.012, x + w * 0.006, y0 + h * 0.012, p.s(ink, 0.35), p.w(0.7)); }
    c.circle(w * 0.47, y0, h * 0.0065, p.s(p.paper, 1.));
    if let Some(o) = PathBuilder::from_circle(w * 0.47, y0, h * 0.0065) { c.stroke(&o, p.s(ink, 1.), p.w(1.2)); }
    let n = 90;
    let hw = |f: f32| h * 0.022 * (1. - 0.28 * f);
    let sori = |f: f32| h * 0.06 * f * f;
    let (mut spine, mut edge, mut ridge, mut temper) = (vec![], vec![], vec![], vec![]);
    for j in 0..=n {
        let f = j as f32 / n as f32;
        let x = w * (0.54 + 0.41 * f);
        let top = y0 - hw(f) - sori(f) - h * 0.001;
        let tip_y = y0 - hw(1.) - sori(1.);
        let k = ((f - 0.9) / 0.1).max(0.).powf(1.6);
        let bot = (y0 + hw(f) - sori(f)) * (1. - k) + tip_y * k;
        spine.push((x, top));
        edge.push((x, bot));
        if f <= 0.9 {
            ridge.push((x, top + hw(f) * 0.62));
            temper.push((x, bot - hw(f) * (0.55 + 0.12 * (f * 70.).sin() + 0.06 * (f * 23.).sin())));
        }
    }
    let mut blade = spine.clone();
    blade.extend(edge.iter().rev());
    sc_shape(c, &sc_poly(&blade, true), fillp.clone(), p.s(ink, 1.), p.w(1.6));
    if let Some(r) = sc_poly(&ridge, false) { c.stroke(&r, p.s(ink, 0.55), p.w(0.9)); }
    if let Some(r) = sc_poly(&temper, false) { c.stroke(&r, p.s(cons, 0.85), p.w(1.1)); }
    let (yx, yr) = (ridge[ridge.len() - 1], edge[(n as f32 * 0.9) as usize]);
    c.line(yx.0, yx.1, yr.0, yr.1, p.s(ink, 0.6), p.w(0.9));
    // 7 the peg, drawn out below its hole along the assembly line
    sc_dash(c, &[(w * 0.47, y0 + h * 0.012), (w * 0.47, y0 + h * 0.075)], &[u * 0.005, u * 0.004], &p.s(cons, 0.75), p.w(0.9));
    sc_shape(c, &sc_rect(w * 0.4675, y0 + h * 0.08, w * 0.005, h * 0.04), fillp.clone(), p.s(ink, 1.), p.w(1.2));
    // the guard face-on, twice size, ringed: the plate, its rim, the openings, the tang's slot and the two side ports
    let (dx, dy, dr) = (w * 0.765, h * 0.255, h * 0.165);
    c.circle(dx, dy, dr, p.s(p.paper, 0.95));
    if let Some(o) = PathBuilder::from_circle(dx, dy, dr) { c.stroke(&o, p.s(ink, 0.7), p.w(1.1)); }
    let pr = h * 0.14;
    let mut rings = vec![sc_ring(dx, dy, pr, pr * 0.94, 0., 96, true)];
    rings.push(vec![(dx - h * 0.009, dy - h * 0.036), (dx - h * 0.006, dy + h * 0.036), (dx + h * 0.006, dy + h * 0.036), (dx + h * 0.009, dy - h * 0.036)]);
    for sd in [-1f32, 1.] { rings.push(sc_ring(dx + sd * h * 0.058, dy, h * 0.014, h * 0.03, 0., 32, false)); }
    for k in 0..8 { let a = (k as f32 + 0.5) / 8. * TAU; rings.push(sc_ring(dx + a.cos() * pr * 0.72, dy + a.sin() * pr * 0.68, h * 0.024, h * 0.009, a + PI / 2., 28, false)); }
    let plate = sc_compound(&rings);
    if let Some(pl) = &plate { sc_hatch(c, pl, PI / 4., u * 0.0055, p.s(ink, 0.38), p.w(0.7)); c.stroke(pl, p.s(ink, 1.), p.w(1.4)); }
    if let Some(rim) = sc_poly(&sc_ring(dx, dy, pr * 0.9, pr * 0.85, 0., 96, true), true) { c.stroke(&rim, p.s(ink, 0.6), p.w(0.9)); }
    sc_text(c, "DETAIL B   2 : 1", &p.disp, p.fs(1.15), dx, dy + dr + p.fs(1.4), 0.5, ink, 0.95);
    let ring_b: Vec<(f32, f32)> = (0..=48).map(|j| { let a = j as f32 / 48. * TAU; (w * 0.354 + a.cos() * h * 0.13, y0 + a.sin() * h * 0.13) }).collect();
    sc_dash(c, &ring_b, &[u * 0.007, u * 0.005], &p.s(mark, 0.8), p.w(1.));
    sc_text(c, "B", &p.disp, p.fs(1.5), w * 0.354 + h * 0.1, y0 - h * 0.1, 0.5, mark, 1.);
    // section C–C through the blade: the ridged profile, cut and hatched
    let (sx, sy, ss) = (w * 0.5, h * 0.25, h * 0.06);
    let prof = sc_poly(&[(sx - ss * 0.16, sy - ss), (sx + ss * 0.16, sy - ss), (sx + ss * 0.3, sy - ss * 0.38), (sx, sy + ss * 1.15), (sx - ss * 0.3, sy - ss * 0.38)], true);
    if let Some(pp) = &prof { c.fill(pp, p.s(p.paper, 1.)); sc_hatch(c, pp, PI / 4., u * 0.0035, p.s(ink, 0.6), p.w(0.7)); }
    sc_shape(c, &prof, None, p.s(ink, 1.), p.w(1.6));
    sc_text(c, "SECTION C–C   4 : 1", &p.disp, p.fs(1.15), sx, sy + ss * 1.15 + p.fs(1.6), 0.5, ink, 0.95);
    let cx = w * 0.72;
    let fcut = (cx / w - 0.54) / 0.41;
    let (ct, cb) = (y0 - hw(fcut) - sori(fcut), y0 + hw(fcut) - sori(fcut));
    for (y, d) in [(ct - h * 0.035, -1f32), (cb + h * 0.035, 1.)] {
        c.line(cx, y, cx, y - d * h * 0.02, p.s(mark, 1.), p.w(2.4));
        sc_arrow(c, &p, &[(cx, y), (cx - w * 0.018, y)], mark, 1., false);
        sc_text(c, "C", &p.disp, p.fs(1.2), cx - w * 0.026, y + p.fs(0.4), 0.5, mark, 1.);
    }
    // measures and the parts list
    sc_dim(c, &p, (w * 0.1, y0 + h * 0.03), (w * 0.3, y0 + h * 0.03), -(h * 0.2), "TSUKA 260", ink);
    sc_dim(c, &p, (w * 0.54, y0 + h * 0.022), (w * 0.95, y0 + h * 0.022), -(h * 0.21), "NAGASA 710", ink);
    let rows = [("1", "KASHIRA", "1", "IRON"), ("2", "TSUKA", "1", "RAYSKIN, SILK"), ("3", "FUCHI", "1", "COPPER"), ("4", "SEPPA", "2", "COPPER"),
        ("5", "TSUBA", "1", "IRON"), ("6", "HABAKI", "1", "COPPER"), ("7", "MEKUGI", "1", "BAMBOO"), ("8", "BLADE", "1", "TAMAHAGANE")];
    let (tx0, ty0, rh) = (w * 0.05, h * 0.085, h * 0.031);
    let cols = [0., w * 0.035, w * 0.13, w * 0.165, w * 0.29];
    sc_shape(c, &sc_rect(tx0, ty0, cols[4], rh * 9.), Some(p.s(p.paper, 0.9)), p.s(ink, 0.9), p.w(1.4));
    for k in 1..9 { c.line(tx0, ty0 + rh * k as f32, tx0 + cols[4], ty0 + rh * k as f32, p.s(ink, if k == 1 { 0.9 } else { 0.35 }), p.w(if k == 1 { 1.3 } else { 0.7 })); }
    for x in &cols[1..4] { c.line(tx0 + x, ty0, tx0 + x, ty0 + rh * 9., p.s(ink, 0.5), p.w(0.8)); }
    let fs = p.fs(0.85);
    for (k, x) in ["ITEM", "PART", "QTY", "MATERIAL"].into_iter().enumerate() { sc_text(c, x, &p.disp, p.fs(1.), tx0 + cols[k] + w * 0.005, ty0 + rh * 0.7, 0., mark, 1.); }
    for (j, r) in rows.iter().enumerate() {
        let y = ty0 + rh * (j as f32 + 1.7);
        for (k, s) in [r.0, r.1, r.2, r.3].into_iter().enumerate() { sc_text(c, s, &p.mono, fs, tx0 + cols[k] + w * 0.005, y, 0., ink, 0.88); }
    }
    for (n, bx, by, ttx, tty) in [("1", 0.075, 0.43, 0.075, 0.524), ("2", 0.2, 0.665, 0.24, 0.578), ("3", 0.29, 0.43, 0.317, 0.518), ("4", 0.337, 0.665, 0.337, 0.6),
        ("5", 0.4, 0.4, 0.359, 0.47), ("6", 0.41, 0.665, 0.4, 0.583), ("7", 0.5, 0.67, 0.4725, 0.66), ("8", 0.62, 0.44, 0.64, 0.523)] {
        sc_balloon(c, &p, w * bx, h * by, w * ttx, h * tty, n, ink);
    }
    sc_sheet(c, &p, "A-04", "DETAIL OF THE BLADE", "1 : 2");
}

/// A-05 Notes: the trace of the star fort on its protractor, the general notes, the revisions and the stamp
fn notes_sheet(c: &mut Cv, t: &Theme) {
    sc_underlay(c, t, (0.2, 0.84, 0.22, 0.2, 0.8), (0.06, 0.32, 0.93, 0.22, 46), 341);
    let p = sc_pen(c, t);
    let (w, h, u) = (c.w, c.h, p.u);
    let (ink, cons, mark) = (p.ink, p.cons, p.mark);
    let (cx, cy, r0) = (w * 0.56, h * 0.47, h * 0.18);
    let rs = r0 * 1.45;
    let v: Vec<(f32, f32)> = (0..5).map(|k| { let a = -PI / 2. + k as f32 * TAU / 5.; (a.cos(), a.sin()) }).collect();
    let at = |d: (f32, f32), r: f32| (cx + d.0 * r, cy + d.1 * r);
    // construction first: the two circles, the capitals, the lines of defence, the protractor
    for rr in [r0, rs] { if let Some(o) = PathBuilder::from_circle(cx, cy, rr) { c.stroke(&o, p.s(cons, 0.4), p.w(0.8)); } }
    let rp = rs * 1.3;
    if let Some(o) = PathBuilder::from_circle(cx, cy, rp) { c.stroke(&o, p.s(cons, 0.75), p.w(1.1)); }
    for k in 0..72 {
        let a = -PI / 2. + k as f32 * TAU / 72.;
        let l = if k % 6 == 0 { u * 0.02 } else if k % 2 == 0 { u * 0.012 } else { u * 0.007 };
        c.line(cx + a.cos() * rp, cy + a.sin() * rp, cx + a.cos() * (rp + l), cy + a.sin() * (rp + l), p.s(cons, 0.75), p.w(0.9));
        // figures read from outside, never upside down
        let mut rot = a + PI / 2.;
        if rot > PI { rot -= TAU; }
        if rot.abs() > PI / 2. { rot += PI; }
        if k % 6 == 0 { sc_turned(c, &format!("{}°", k * 5), &p.mono, p.fs(0.75), cx + a.cos() * (rp + u * 0.03), cy + a.sin() * (rp + u * 0.03), rot, cons, 0.9); }
    }
    // the trace: for each side of the pentagon, the bastion's face, its flank, the curtain, the next flank
    let mut outline = vec![];
    let mut flanks = vec![];
    let mut sides = vec![];
    for k in 0..5 {
        let (a, b) = (at(v[k], r0), at(v[(k + 1) % 5], r0));
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let el = (ex * ex + ey * ey).sqrt();
        let (mx, my) = ((a.0 + b.0) / 2. - cx, (a.1 + b.1) / 2. - cy);
        let ml = (mx * mx + my * my).sqrt();
        let o = (mx / ml, my / ml);
        let fl = r0 * 0.2;
        let b1 = (a.0 + ex * 0.3, a.1 + ey * 0.3);
        let b2 = (a.0 + ex * 0.7, a.1 + ey * 0.7);
        let a1 = (b1.0 + o.0 * fl, b1.1 + o.1 * fl);
        let a2 = (b2.0 + o.0 * fl, b2.1 + o.1 * fl);
        outline.extend([at(v[k], rs), a1, b1, b2, a2]);
        flanks.push((b1, b2));
        sides.push((a1, b1, b2, a2, o));
        c.line(cx, cy, at(v[k], rs).0, at(v[k], rs).1, p.s(cons, 0.4), p.w(0.8));
        // the ravelin out in front of this curtain
        let (m0, ee) = (((a.0 + b.0) / 2., (a.1 + b.1) / 2.), (ex / el, ey / el));
        let rav = [(m0.0 + o.0 * r0 * 0.8, m0.1 + o.1 * r0 * 0.8), (m0.0 + o.0 * r0 * 0.36 + ee.0 * r0 * 0.27, m0.1 + o.1 * r0 * 0.36 + ee.1 * r0 * 0.27),
            (m0.0 + o.0 * r0 * 0.36 - ee.0 * r0 * 0.27, m0.1 + o.1 * r0 * 0.36 - ee.1 * r0 * 0.27)];
        let rp_ = sc_poly(&rav, true);
        if let Some(q) = &rp_ { c.fill(q, p.s(p.paper, 0.9)); sc_hatch(c, q, PI / 4., u * 0.005, p.s(ink, 0.4), p.w(0.7)); }
        sc_shape(c, &rp_, None, p.s(ink, 0.95), p.w(1.3));
    }
    for k in 0..5 {
        let (b1, b2) = flanks[k];
        let (sa, sb) = (at(v[(k + 1) % 5], rs), at(v[k], rs));
        sc_dash(c, &[b1, sb], &[u * 0.006, u * 0.005], &p.s(cons, 0.5), p.w(0.8));
        sc_dash(c, &[b2, sa], &[u * 0.006, u * 0.005], &p.s(cons, 0.5), p.w(0.8));
    }
    // the ditch, wet, between the walls and the counterscarp
    let scaled = |f: f32| -> Vec<(f32, f32)> { outline.iter().map(|&(x, y)| (cx + (x - cx) * f, cy + (y - cy) * f)).collect() };
    let mut ditch = vec![scaled(1.14)];
    let mut inner = outline.clone();
    inner.reverse();
    ditch.push(inner);
    if let Some(d) = sc_compound(&ditch) { c.fill(&d, p.s(cons, 0.12)); }
    if let Some(cs) = sc_poly(&scaled(1.14), true) { c.stroke(&cs, p.s(ink, 0.6), p.w(1.)); }
    // the bastions solid with earth and hatched; the curtains a rampart, its inner line drawn
    for k in 0..5 {
        let (a1, b1, _, _, _) = sides[k];
        let (_, _, b2, a2, _) = sides[(k + 4) % 5];
        let bast = sc_poly(&[a2, at(v[k], rs), a1, b1, b2], true);
        if let Some(bp) = &bast { c.fill(bp, p.s(p.paper, 0.85)); sc_hatch(c, bp, PI / 4., u * 0.0048, p.s(ink, 0.45), p.w(0.7)); }
    }
    for &(_, b1, b2, _, o) in &sides { let d = r0 * 0.08; c.line(b1.0 - o.0 * d, b1.1 - o.1 * d, b2.0 - o.0 * d, b2.1 - o.1 * d, p.s(ink, 0.8), p.w(1.1)); }
    sc_shape(c, &sc_poly(&outline, true), None, p.s(ink, 1.), p.w(2.));
    // the angles that matter
    let (a0, a1) = (-PI / 2., -PI / 2. + TAU / 5.);
    if let Some(ar) = Cv::arc(cx, cy, r0 * 0.3, r0 * 0.3, a0, a1) { c.stroke(&ar, p.s(mark, 0.95), p.w(1.2)); }
    let am = (a0 + a1) / 2.;
    sc_text(c, "72°", &p.mono, p.fs(0.9), cx + am.cos() * r0 * 0.42, cy + am.sin() * r0 * 0.42 + p.fs(0.3), 0.5, mark, 1.);
    let s0 = at(v[0], rs);
    let (q1, q2) = (outline[1], outline[19]);
    let (g1, g2) = ((q1.1 - s0.1).atan2(q1.0 - s0.0), (q2.1 - s0.1).atan2(q2.0 - s0.0));
    let (lo, hi) = (g1.min(g2), g1.max(g2));
    if let Some(ar) = Cv::arc(s0.0, s0.1, r0 * 0.22, r0 * 0.22, lo, hi) { c.stroke(&ar, p.s(mark, 0.95), p.w(1.2)); }
    sc_text(c, &format!("{:.0}°", (hi - lo).to_degrees()), &p.mono, p.fs(0.9), s0.0, s0.1 + r0 * 0.36, 0.5, mark, 1.);
    sc_text(c, "TRACE OF THE STAR FORT   1 : 2000", &p.disp, p.fs(1.2), cx, cy + rp + u * 0.05, 0.5, ink, 0.9);
    // the notes, the revisions, the stamp
    sc_notes(c, &p, w * 0.055, h * 0.12, "GENERAL NOTES", &["1. ALL DIMENSIONS IN PACES", "   UNLESS NOTED OTHERWISE.", "2. DO NOT SCALE FROM DRAWING.",
        "3. FLANKS BEFORE FACES.", "4. EVERY FACE IS SEEN FROM", "   A FLANK: NO DEAD GROUND.", "5. DITCH TO BE KEPT WET.", "6. VERIFY ON SITE BEFORE", "   THE FIRST SHOT."]);
    let (rx, ry, rw, rh) = (w * 0.765, h * 0.075, w * 0.19, h * 0.03);
    sc_shape(c, &sc_rect(rx, ry, rw, rh * 4.), Some(p.s(p.paper, 0.9)), p.s(ink, 0.9), p.w(1.3));
    let rc = [0., rw * 0.12, rw * 0.32, rw * 0.86];
    for k in 1..4 { c.line(rx, ry + rh * k as f32, rx + rw, ry + rh * k as f32, p.s(ink, if k == 1 { 0.85 } else { 0.35 }), p.w(0.8)); }
    for x in &rc[1..] { c.line(rx + x, ry, rx + x, ry + rh * 4., p.s(ink, 0.5), p.w(0.8)); }
    for (k, s) in ["REV", "DATE", "DESCRIPTION", "BY"].into_iter().enumerate() { sc_text(c, s, &p.disp, p.fs(0.9), rx + rc[k] + w * 0.004, ry + rh * 0.7, 0., mark, 1.); }
    for (j, r) in [["A", "01.03", "ISSUED FOR SIEGE", "RS"], ["B", "14.03", "RAVELINS ADDED", "RS"], ["C", "02.04", "AS BUILT", "RS"]].iter().enumerate() {
        for (k, s) in r.iter().enumerate() { sc_text(c, s, &p.mono, p.fs(0.78), rx + rc[k] + w * 0.004, ry + rh * (j as f32 + 1.7), 0., ink, 0.88); }
    }
    let (sx, sy) = (w * 0.17, h * 0.75);
    let keep = c.tf;
    c.tf = Transform::from_rotate(-9.).post_translate(sx, sy);
    let (bw, bh) = (u * 0.11, u * 0.04);
    sc_shape(c, &sc_rect(-bw / 2., -bh / 2., bw, bh), None, p.s(mark, 0.85), p.w(2.4));
    sc_shape(c, &sc_rect(-bw / 2. + u * 0.003, -bh / 2. + u * 0.003, bw - u * 0.006, bh - u * 0.006), None, p.s(mark, 0.75), p.w(1.));
    sc_text(c, "APPROVED", &p.disp, p.fs(2.), 0., p.fs(0.25), 0.5, mark, 0.9);
    sc_text(c, "FOR THE SIEGE  ·  REV C", &p.mono, p.fs(0.6), 0., bh / 2. - u * 0.006, 0.5, mark, 0.85);
    c.tf = keep;
    sc_sheet(c, &p, "A-05", "NOTES AND THE TRACE", "AS SHOWN");
}

fn schematic_finish(c: &mut Cv, t: &Theme, seed: u32) {
    let (w, h) = (c.w, c.h);
    let p = t.palette;
    let lw = (w.max(h) * 0.0025).max(2.);
    // folded in four to travel in a coat: a pale ridge with a shadow along each crease
    c.line(w * 0.5, 0., w * 0.5, h, Cv::solid(col(p.ink, 0.045)), lw);
    c.line(w * 0.5 + lw * 0.8, 0., w * 0.5 + lw * 0.8, h, Cv::solid(col(BLACK, 0.12)), lw * 0.5);
    c.line(0., h * 0.5, w, h * 0.5, Cv::solid(col(p.ink, 0.04)), lw);
    c.line(0., h * 0.5 + lw * 0.8, w, h * 0.5 + lw * 0.8, Cv::solid(col(BLACK, 0.1)), lw * 0.5);
    c.vignette(BLACK, 0.42);
    c.noise(0.045, p.ink, seed);
    c.noise(0.05, BLACK, seed ^ 0x51ed);
}

fn schematic(c: &mut Cv, t: &Theme, i: usize) {
    match i {
        0 => plan_sheet(c, t),
        1 => section_sheet(c, t),
        2 => elevation_sheet(c, t),
        3 => detail_sheet(c, t),
        _ => notes_sheet(c, t),
    }
    c.reset();
    schematic_finish(c, t, 110 + i as u32);
}

/* ── Marquee: a crime picture in five acts, as cut-paper posters ───────────────────────────────────────── */
// Red, black and paper and nothing else. Every shape is cut with scissors (each edge wanders a hair), lifted off
// the sheet by a sliver of shadow, and screen-printed with the odd dropout. The city at night, a stranger under a
// lamp, the road out of town, the fall, and the last reel. The same hat turns up in every act.

/// the poster's three inks and its type
struct Bill { u: f32, red: Rgb, ink: Rgb, paper: Rgb, disp: String, body: String, mono: String }
fn mq_bill(c: &Cv, t: &Theme) -> Bill {
    let p = t.palette;
    let (ink, paper) = if p.ink.luma() <= p.bg.luma() { (p.ink, p.bg) } else { (p.bg, p.ink) };
    Bill { u: c.h.min(c.w * 0.5625), red: p.accent, ink, paper, disp: t.display.clone(), body: t.body.clone(), mono: t.mono.clone() }
}

/// a polygon cut with scissors: each edge is a few straight snips that wander off the line by up to `wob`
fn mq_cut(pts: &[(f32, f32)], wob: f32, r: &mut Rng) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    let n = pts.len();
    for k in 0..n {
        let (a, z) = (pts[k], pts[(k + 1) % n]);
        let (dx, dy) = (z.0 - a.0, z.1 - a.1);
        let len = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = if len > 0. { (-dy / len, dx / len) } else { (0., 0.) };
        let steps = ((len / (wob * 16.).max(1.)) as usize).clamp(1, 40);
        for s in 0..steps {
            let f = s as f32 / steps as f32;
            let j = if s == 0 { 0. } else { (r.f() - 0.5) * wob };
            let (x, y) = (a.0 + dx * f + nx * j, a.1 + dy * f + ny * j);
            if k == 0 && s == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
        }
    }
    b.close();
    b.finish()
}
/// paste one cut piece: its shadow first (lifted by `lift`), then the paper
fn mq_paste(c: &mut Cv, b: &Bill, pts: &[(f32, f32)], tone: Rgb, a: f32, lift: f32, r: &mut Rng) {
    if let Some(path) = mq_cut(pts, b.u * 0.0024, r) {
        if lift > 0. {
            let d = b.u * 0.0035 * lift;
            if let Some(s) = path.clone().transform(Transform::from_translate(d * 0.6, d)) { c.fill(&s, Cv::solid(col(BLACK, 0.3 * a))); }
        }
        c.fill(&path, Cv::solid(col(tone, a)));
    }
}
fn mq_box(x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] { [(x, y), (x + w, y), (x + w, y + h), (x, y + h)] }
fn mq_poly(pts: &[(f32, f32)]) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    for (k, &(x, y)) in pts.iter().enumerate() { if k == 0 { b.move_to(x, y) } else { b.line_to(x, y) } }
    b.close();
    b.finish()
}
fn mq_disc(x: f32, y: f32, rx: f32, ry: f32, n: usize) -> Vec<(f32, f32)> {
    (0..n).map(|k| { let a = k as f32 / n as f32 * TAU; (x + a.cos() * rx, y + a.sin() * ry) }).collect()
}

/// a flooded field of one ink, laid a little unevenly as a big screen-print is
fn mq_ground(c: &mut Cv, tone: Rgb, seed: u64) {
    let (w, h) = (c.w, c.h);
    c.fill_all(Cv::solid(col(tone, 1.)));
    let mut r = Rng::new(seed);
    for _ in 0..24 {
        let (x, y, rad) = (r.f() * w, r.f() * h, (0.12 + r.f() * 0.28) * w.max(h));
        c.cloud(x, y, rad, if r.f() < 0.55 { BLACK } else { WHITE }, 0.04);
    }
}
/// text set from its left edge, its centre or its right edge (align 0, 0.5, 1); never wider than `max`
fn mq_set(c: &mut Cv, s: &str, fam: &str, size: f32, x: f32, y: f32, align: f32, max: f32, tone: Rgb, a: f32) -> f32 {
    let w0 = crate::paint::measure(s, fam, size);
    let size = if w0 > max && w0 > 0. { size * max / w0 } else { size };
    let wd = crate::paint::measure(s, fam, size);
    c.text(s, fam, size, x - wd * align, y, col(tone, a));
    size
}
/// the act card: a small act number over a rule, the title under it in the display face
fn mq_titles(c: &mut Cv, b: &Bill, act: &str, title: &str, x: f32, y: f32, size: f32, act_tone: Rgb, title_tone: Rgb) {
    let u = b.u;
    let s = mq_set(c, act, &b.body, size * 0.26, x, y - size * 1.02, 0., u, act_tone, 1.);
    let aw = crate::paint::measure(act, &b.body, s);
    c.fill_rect(x + aw + u * 0.012, y - size * 1.02 - s * 0.36, u * 0.09, u * 0.004, Cv::solid(col(act_tone, 1.)));
    mq_set(c, title, &b.disp, size, x, y, 0., c.w * 0.6, title_tone, 1.);
}
/// the billing block: tall condensed credits stacked to one width, as at the foot of a one-sheet
fn mq_billing(c: &mut Cv, b: &Bill, cx: f32, y: f32, wid: f32, tone: Rgb, a: f32) {
    let u = b.u;
    let s1 = mq_set(c, "A RICE STUDIO PICTURE", &b.body, u * 0.017, cx, y, 0.5, wid * 0.7, tone, a);
    let y2 = y + s1 * 1.35;
    let s2 = mq_set(c, "DIRECTED BY YOU  ·  SCREENPLAY WRITTEN IN CODE  ·  MUSIC BY THE CITY AFTER DARK", &b.body, u * 0.013, cx, y2, 0.5, wid, tone, a * 0.85);
    let y3 = y2 + s2 * 1.5;
    mq_set(c, "IN RED, BLACK AND PAPER  ·  NOW SHOWING", &b.mono, u * 0.0105, cx, y3, 0.5, wid * 0.8, tone, a * 0.7);
}

/// the stranger, in unit coordinates (feet at the origin, y up, one unit = his height): coat, legs, head, hat
fn mq_stranger_parts() -> Vec<Vec<(f32, f32)>> {
    vec![
        // the trench coat, collar up, belted, a little flare at the hem
        vec![(-0.135, 0.27), (-0.112, 0.515), (-0.127, 0.56), (-0.142, 0.74), (-0.12, 0.8), (-0.062, 0.83), (-0.068, 0.885), (-0.03, 0.862),
             (0.03, 0.862), (0.068, 0.885), (0.062, 0.83), (0.12, 0.8), (0.142, 0.74), (0.127, 0.56), (0.112, 0.515), (0.135, 0.27)],
        // legs and shoes
        vec![(-0.072, 0.28), (-0.022, 0.28), (-0.026, 0.012), (-0.01, 0.0), (-0.09, 0.0), (-0.074, 0.016)],
        vec![(0.022, 0.28), (0.072, 0.28), (0.07, 0.016), (0.1, 0.0), (0.02, 0.0), (0.026, 0.012)],
        // the head, under the hat
        mq_disc(0., 0.895, 0.046, 0.05, 18),
        // fedora: the crown with its pinch, then the brim snapped down at the front
        vec![(-0.052, 0.93), (-0.047, 0.983), (-0.012, 1.0), (0., 0.99), (0.012, 1.0), (0.047, 0.983), (0.052, 0.93)],
        vec![(-0.108, 0.928), (0.108, 0.936), (0.1, 0.948), (-0.1, 0.944)],
    ]
}
/// a fedora on its own, in unit coordinates (brim width 1, y down, centre of the brim at the origin)
fn mq_hat(c: &mut Cv, b: &Bill, x: f32, y: f32, s: f32, rot: f32, tone: Rgb, band: Rgb, r: &mut Rng) {
    let (ca, sa) = (rot.cos(), rot.sin());
    let m = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> { pts.iter().map(|&(px, py)| (x + (px * ca - py * sa) * s, y + (px * sa + py * ca) * s)).collect() };
    mq_paste(c, b, &m(&[(-0.5, 0.0), (-0.3, -0.06), (0.3, -0.06), (0.5, 0.0), (0.42, 0.07), (-0.42, 0.07)]), tone, 1., 1., r);
    mq_paste(c, b, &m(&[(-0.25, -0.04), (-0.22, -0.32), (-0.05, -0.4), (0., -0.35), (0.05, -0.4), (0.22, -0.32), (0.25, -0.04)]), tone, 1., 0.6, r);
    mq_paste(c, b, &m(&[(-0.245, -0.05), (0.245, -0.05), (0.238, -0.12), (-0.238, -0.12)]), band, 1., 0., r);
}

/// Act I: the city at night. A red sky, a paper moon, searchlights, and in one lit window a man in a hat
fn mq_city(c: &mut Cv, b: &Bill) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(601);
    mq_ground(c, b.red, 601);
    c.fill_all(Cv::linear(0., 0., 0., h, &[(0., col(BLACK, 0.34)), (0.6, col(BLACK, 0.)), (1., col(BLACK, 0.))]));
    // two searchlights from the street, swinging across the roofs (drawn before the moon, which they pass behind)
    for &(x0, ang, a) in &[(w * 0.36, -1.12_f32, 0.5_f32), (w * 0.9, -1.98, 0.36)] {
        let (len, sp) = (h * 1.5, 0.045);
        let p = [(x0, h * 0.86), (x0 + (ang - sp).cos() * len, h * 0.86 + (ang - sp).sin() * len), (x0 + (ang + sp).cos() * len, h * 0.86 + (ang + sp).sin() * len)];
        if let Some(path) = mq_poly(&p) {
            c.fill(&path, Cv::linear(p[0].0, p[0].1, (p[1].0 + p[2].0) / 2., (p[1].1 + p[2].1) / 2., &[(0., col(b.paper, a)), (0.85, col(b.paper, 0.))]));
        }
    }
    mq_paste(c, b, &mq_disc(w * 0.72, h * 0.31, u * 0.19, u * 0.19, 48), b.paper, 1., 1.4, &mut r);
    // the back row in the red's own shadow; the front row in black, with windows
    let back = b.red.mix(BLACK, 0.52);
    let mut front: Vec<(f32, f32, f32)> = vec![];
    for (row, (lo, hi, tone)) in [(0.40_f32, 0.62_f32, back), (0.5, 0.78, b.ink)].into_iter().enumerate() {
        let mut x = -u * 0.03 * r.f();
        while x < w {
            let bw = u * (0.07 + r.f() * 0.1);
            let top = h * (lo + r.f() * (hi - lo));
            let (x0, x1) = (x, x + bw);
            let pts: Vec<(f32, f32)> = match (r.f() * 5.) as u32 {
                // a setback tower
                0 | 1 => { let (ins, st) = (bw * 0.18, u * (0.04 + r.f() * 0.05)); vec![(x0 + ins, top), (x1 - ins, top), (x1 - ins, top + st), (x1, top + st), (x1, h), (x0, h), (x0, top + st), (x0 + ins, top + st)] }
                // a gable
                2 => vec![((x0 + x1) / 2., top - u * 0.035), (x1, top), (x1, h), (x0, h), (x0, top)],
                _ => vec![(x0, top), (x1, top), (x1, h), (x0, h)],
            };
            mq_paste(c, b, &pts, tone, 1., if row == 1 { 1. } else { 0.5 }, &mut r);
            // an aerial or a water tank on some roofs
            let k = r.f();
            if k < 0.22 { c.line((x0 + x1) / 2., top, (x0 + x1) / 2., top - u * (0.06 + r.f() * 0.06), Cv::solid(col(tone, 1.)), u * 0.004); }
            else if k < 0.36 {
                let (tx, tw) = (x0 + bw * (0.2 + r.f() * 0.4), u * 0.026);
                mq_paste(c, b, &[(tx, top - u * 0.05), (tx + tw, top - u * 0.05), (tx + tw * 1.08, top - u * 0.018), (tx - tw * 0.08, top - u * 0.018)], tone, 1., 0., &mut r);
                for lx in [tx + tw * 0.1, tx + tw * 0.9] { c.line(lx, top - u * 0.02, lx, top, Cv::solid(col(tone, 1.)), u * 0.003); }
            }
            if row == 1 { front.push((x0, x1, top)); }
            x = x1 + u * r.f() * 0.012;
        }
    }
    // the windows: most dark, a few lit paper, one or two red
    let (gx, gy) = (u * 0.019, u * 0.027);
    for &(x0, x1, top) in &front {
        let mut wy = top + u * 0.05;
        while wy < h * 0.86 {
            let mut wx = x0 + u * 0.012;
            while wx + u * 0.009 < x1 - u * 0.008 {
                let k = r.f();
                if k < 0.12 { c.fill_rect(wx, wy, u * 0.0085, u * 0.013, Cv::solid(col(b.paper, 0.92))); }
                else if k < 0.145 { c.fill_rect(wx, wy, u * 0.0085, u * 0.013, Cv::solid(col(b.red, 0.95))); }
                wx += gx;
            }
            wy += gy;
        }
    }
    // the one window that matters: wide, lit, and somebody in it wearing the hat
    if let Some(&(x0, x1, top)) = front.iter().min_by(|a, z| ((a.0 + a.1) / 2. - w * 0.47).abs().total_cmp(&((z.0 + z.1) / 2. - w * 0.47).abs())) {
        let (wx, wy, ww, wh) = ((x0 + x1) / 2. - u * 0.026, top + u * 0.09, u * 0.052, u * 0.07);
        mq_paste(c, b, &mq_box(wx, wy, ww, wh), b.paper, 1., 0., &mut r);
        let (fx, fy, fs) = (wx + ww * 0.5, wy + wh, wh * 0.95);
        for part in mq_stranger_parts().iter().skip(3) {
            let pts: Vec<_> = part.iter().map(|&(px, py)| (fx + px * fs * 1.9, fy - (py - 0.62) * fs * 1.9)).collect();
            if let Some(p) = mq_poly(&pts) { c.fill(&p, Cv::solid(col(b.ink, 1.))); }
        }
        let sh = [(fx - fs * 0.27, fy), (fx - fs * 0.2, fy - fs * 0.38), (fx, fy - fs * 0.44), (fx + fs * 0.2, fy - fs * 0.38), (fx + fs * 0.27, fy)];
        if let Some(p) = mq_poly(&sh) { c.fill(&p, Cv::solid(col(b.ink, 1.))); }
        c.fill_rect(wx + ww * 0.48, wy, ww * 0.04, wh, Cv::solid(col(b.ink, 1.)));
    }
    // the street, cut on a slant across the foot of the sheet
    mq_paste(c, b, &[(-u * 0.02, h * 0.855), (w + u * 0.02, h * 0.9), (w + u * 0.02, h + u * 0.02), (-u * 0.02, h + u * 0.02)], b.ink, 1., 1.5, &mut r);
    c.line(-u * 0.02, h * 0.855 - u * 0.006, w + u * 0.02, h * 0.9 - u * 0.006, Cv::solid(col(b.paper, 0.9)), u * 0.004);
    mq_titles(c, b, "ACT  I", "THE CITY", w * 0.055, h * 0.36, u * 0.16, b.ink, b.paper);
    mq_billing(c, b, w * 0.27, h * 0.935, u * 0.62, b.paper, 0.85);
}

/// Act II: the stranger under a lamp. Night, one cone of paper light, rain in it, and his shadow laid out in red
fn mq_stranger(c: &mut Cv, b: &Bill) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(602);
    mq_ground(c, b.ink, 602);
    let gy = h * 0.82;
    let (lx, ly) = (w * 0.7, h * 0.25);
    // a brick wall at the back, barely there: courses of black on black
    for k in 0..((gy / (u * 0.03)) as usize) {
        let y = k as f32 * u * 0.03;
        c.line(0., y, w, y, Cv::solid(col(b.paper, 0.025)), u * 0.0018);
    }
    // the cone of light and its pool on the pavement
    let cone = [(lx - u * 0.035, ly + u * 0.02), (lx + u * 0.035, ly + u * 0.02), (lx + u * 0.44, gy + u * 0.03), (lx - u * 0.44, gy + u * 0.03)];
    mq_paste(c, b, &cone, b.paper, 1., 0., &mut r);
    mq_paste(c, b, &mq_disc(lx, gy + u * 0.03, u * 0.44, u * 0.06, 40), b.paper, 1., 0., &mut r);
    // rain, only where the light catches it
    if let Some(cp) = mq_poly(&cone) {
        c.clip_path(&cp);
        for _ in 0..170 {
            let (x, y, l) = (lx + (r.f() - 0.5) * u * 0.9, ly + r.f() * (gy - ly), u * (0.02 + r.f() * 0.035));
            c.line(x, y, x - l * 0.28, y + l, Cv::solid(col(b.ink, 0.55 + r.f() * 0.35)), u * 0.0022);
        }
        c.clip = None;
    }
    // the lamp: post, arm and shade
    c.fill_rect(lx + u * 0.075, ly - u * 0.02, u * 0.012, gy - ly + u * 0.02, Cv::solid(col(b.ink, 1.)));
    mq_paste(c, b, &[(lx - u * 0.05, ly + u * 0.024), (lx + u * 0.05, ly + u * 0.024), (lx + u * 0.028, ly - u * 0.012), (lx - u * 0.028, ly - u * 0.012)], b.ink, 1., 0., &mut r);
    c.line(lx, ly - u * 0.012, lx + u * 0.081, ly - u * 0.035, Cv::solid(col(b.ink, 1.)), u * 0.008);
    c.circle(lx, ly + u * 0.026, u * 0.012, Cv::solid(col(b.red, 1.)));
    // the man, and his shadow thrown long to the left across the pavement
    let (fx, fs) = (lx - u * 0.16, u * 0.42);
    for part in mq_stranger_parts() {
        let sh: Vec<_> = part.iter().map(|&(px, py)| (fx + px * fs * 0.4 - py * fs * 2.1, gy + py * fs * 0.2 + px * fs * 0.9)).collect();
        if let Some(p) = mq_poly(&sh) { c.fill(&p, Cv::solid(col(b.red, 1.))); }
    }
    for part in mq_stranger_parts() {
        let pts: Vec<_> = part.iter().map(|&(px, py)| (fx + px * fs, gy - py * fs)).collect();
        mq_paste(c, b, &pts, b.ink, 1., 0.8, &mut r);
    }
    // a hatband of red, and the cigarette
    let (hx, hy) = (fx, gy - 0.936 * fs);
    c.fill_rect(hx - 0.05 * fs, hy - 0.012 * fs, 0.1 * fs, 0.012 * fs, Cv::solid(col(b.red, 1.)));
    c.line(fx + 0.04 * fs, gy - 0.872 * fs, fx + 0.07 * fs, gy - 0.866 * fs, Cv::solid(col(b.paper, 1.)), u * 0.003);
    c.circle(fx + 0.071 * fs, gy - 0.866 * fs, u * 0.0035, Cv::solid(col(b.red, 1.)));
    mq_titles(c, b, "ACT  II", "THE STRANGER", w * 0.055, h * 0.36, u * 0.14, b.paper, b.red);
    mq_billing(c, b, w * 0.36, h * 0.955, u * 0.56, b.paper, 0.8);
}

/// Act III: the road out of town. A red dusk, a paper sun on the horizon, the poles, and the car leaving
fn mq_road(c: &mut Cv, b: &Bill) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(603);
    mq_ground(c, b.red, 603);
    let hy = h * 0.57;
    c.fill_all(Cv::linear(0., 0., 0., hy, &[(0., col(BLACK, 0.4)), (1., col(BLACK, 0.))]));
    mq_paste(c, b, &mq_disc(w * 0.66, hy, u * 0.17, u * 0.17, 48), b.paper, 1., 0., &mut r);
    // the land: low hills along the horizon, then all black
    let mut land = vec![(-u * 0.02, h + u * 0.02)];
    let mut x = -u * 0.02;
    while x < w + u * 0.05 { land.push((x, hy - u * 0.012 * (1. + (x / w * 9.).sin()) * r.f().max(0.4))); x += u * 0.06; }
    land.push((w + u * 0.02, h + u * 0.02));
    mq_paste(c, b, &land, b.ink, 1., 1., &mut r);
    // the road, cut from one strip of paper, bending away to the vanishing point
    let vx = w * 0.6;
    let at = |d: f32| -> (f32, f32, f32) { let y = hy + (h - hy) * d; (vx - w * 0.15 * d + w * 0.3 * d * (1. - d) * (1. - d), y, w * 0.6 * d + u * 0.003) };
    let n = 40;
    let mut road = vec![];
    let dd = |k: usize| 1.06 * (k as f32 / n as f32).powf(1.6);
    for k in 0..=n { let (cx, y, wd) = at(dd(k)); road.push((cx - wd / 2., y)); }
    for k in (0..=n).rev() { let (cx, y, wd) = at(dd(k)); road.push((cx + wd / 2., y)); }
    if let Some(p) = mq_cut(&road, u * 0.002, &mut r) { c.fill(&p, Cv::solid(col(b.paper, 1.))); }
    // red dashes down the middle, longer as they come nearer
    let mut d = 0.04_f32;
    while d < 1. {
        let d2 = d * 1.12;
        let (c1, y1, w1) = at(d);
        let (c2, y2, w2) = at(d2.min(1.));
        if let Some(p) = mq_poly(&[(c1 - w1 * 0.008, y1), (c1 + w1 * 0.008, y1), (c2 + w2 * 0.008, y2), (c2 - w2 * 0.008, y2)]) { c.fill(&p, Cv::solid(col(b.red, 1.))); }
        d = d2 * 1.14;
    }
    // telephone poles marching off along the right verge, wires sagging between them
    let mut tops = vec![];
    let mut d = 0.98_f32;
    while d > 0.035 {
        let (cx, y, wd) = at(d);
        let (px, ph, lw) = (cx + wd * 0.66, (h - hy) * d * 1.25 + u * 0.01, (u * 0.016 * d).max(1.2));
        c.line(px, y, px, y - ph, Cv::solid(col(b.ink, 1.)), lw);
        c.line(px - ph * 0.14, y - ph * 0.92, px + ph * 0.14, y - ph * 0.92, Cv::solid(col(b.ink, 1.)), lw * 0.8);
        tops.push((px - ph * 0.12, px + ph * 0.12, y - ph * 0.92, ph));
        d *= 0.74;
    }
    for pair in tops.windows(2) {
        let (a, z) = (pair[0], pair[1]);
        for (xa, xz) in [(a.0, z.0), (a.1, z.1)] {
            let mut pb = PathBuilder::new();
            pb.move_to(xa, a.2);
            pb.quad_to((xa + xz) / 2., (a.2 + z.2) / 2. + a.3 * 0.08, xz, z.2);
            if let Some(p) = pb.finish() { c.stroke(&p, Cv::solid(col(b.ink, 1.)), (u * 0.002).max(1.)); }
        }
    }
    // the car, from behind, leaving: fins, tail-lights, chrome, and the plate
    let (cx, cy, cw) = (at(0.9).0 + at(0.9).2 * 0.22, h * 0.965, w * 0.27);
    let m = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> { pts.iter().map(|&(px, py)| (cx + px * cw, cy - py * cw)).collect() };
    for wx in [-0.43_f32, 0.29] { mq_paste(c, b, &m(&[(wx, 0.08), (wx + 0.14, 0.08), (wx + 0.14, -0.03), (wx, -0.03)]), b.ink, 1., 0., &mut r); }
    mq_paste(c, b, &m(&[(-0.3, 0.4), (0.3, 0.4), (0.235, 0.63), (-0.235, 0.63)]), b.ink, 1., 1., &mut r);
    mq_paste(c, b, &m(&[(-0.5, 0.05), (-0.52, 0.34), (-0.49, 0.45), (-0.43, 0.39), (0.43, 0.39), (0.49, 0.45), (0.52, 0.34), (0.5, 0.05)]), b.ink, 1., 1.2, &mut r);
    mq_paste(c, b, &m(&[(-0.25, 0.43), (0.25, 0.43), (0.205, 0.6), (-0.205, 0.6)]), b.red.mix(BLACK, 0.3), 1., 0., &mut r);
    for s in [-1_f32, 1.] { mq_paste(c, b, &m(&[(0.505 * s, 0.2), (0.44 * s, 0.2), (0.45 * s, 0.37), (0.495 * s, 0.4)]), b.red, 1., 0., &mut r); }
    mq_paste(c, b, &m(&[(-0.49, 0.085), (0.49, 0.085), (0.49, 0.115), (-0.49, 0.115)]), b.paper, 1., 0., &mut r);
    mq_paste(c, b, &m(&[(-0.085, 0.14), (0.085, 0.14), (0.085, 0.235), (-0.085, 0.235)]), b.paper, 1., 0., &mut r);
    mq_set(c, "ACT III", &b.mono, cw * 0.05, cx, cy - cw * 0.165, 0.5, cw * 0.15, b.ink, 1.);
    // the hat on the parcel shelf behind the rear window
    mq_hat(c, b, cx + cw * 0.1, cy - cw * 0.455, cw * 0.15, 0.08, b.ink, b.red, &mut r);
    mq_titles(c, b, "ACT  III", "THE ROAD", w * 0.055, h * 0.36, u * 0.16, b.ink, b.paper);
    mq_billing(c, b, w * 0.055 + u * 0.3, h * 0.43, u * 0.6, b.ink, 0.8);
}

/// Act IV: the fall. Straight down between four towers to the red street; a man in paper tumbling, his hat away
fn mq_fall(c: &mut Cv, b: &Bill) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(604);
    mq_ground(c, b.red, 604);
    let (vx, vy) = (w * 0.5, h * 0.5);
    let (ix, iy) = (u * 0.13, u * 0.075);
    let inner = [(vx - ix, vy - iy), (vx + ix, vy - iy), (vx + ix, vy + iy), (vx - ix, vy + iy)];
    let outer = [(-1., -1.), (w + 1., -1.), (w + 1., h + 1.), (-1., h + 1.)];
    let lerp = |a: (f32, f32), z: (f32, f32), t: f32| (a.0 + (z.0 - a.0) * t, a.1 + (z.1 - a.1) * t);
    // floors fall away evenly in depth, so on the sheet they crowd towards the street
    let z1 = (w / 2.) / ix;
    let g = |d: f32| { let z = 1. + d * (z1 - 1.); (1. - 1. / z) / (1. - 1. / z1) };
    let tones = [b.ink.mix(b.paper, 0.1), b.ink, b.ink.mix(b.paper, 0.05), b.ink.mix(b.red, 0.12)];
    // the street far below: red, with paper crossings and tiny black cars
    for k in 0..7 { c.fill_rect(vx - ix * 0.95 + k as f32 * ix * 0.27, vy - iy * 0.1, ix * 0.12, iy * 0.2, Cv::solid(col(b.paper, 0.85))); }
    for _ in 0..7 { c.fill_rect(vx + (r.f() - 0.5) * ix * 1.8, vy + (r.f() - 0.5) * iy * 1.6, ix * 0.07, iy * 0.11, Cv::solid(col(b.ink, 1.))); }
    for f in 0..4 {
        let (a0, a1, b0, b1) = (outer[f], outer[(f + 1) % 4], inner[f], inner[(f + 1) % 4]);
        mq_paste(c, b, &[a0, a1, b1, b0], tones[f], 1., 0., &mut r);
        let cols = if f % 2 == 0 { 9 } else { 6 };
        let floors = 34;
        for fl in 0..floors {
            let (d0, d1) = (g((fl as f32 + 0.3) / floors as f32), g((fl as f32 + 0.75) / floors as f32));
            for cl in 0..cols {
                let k = r.f();
                let tone = if k < 0.16 { b.paper } else if k < 0.2 { b.red } else { continue };
                let (s0, s1) = ((cl as f32 + 0.22) / cols as f32, (cl as f32 + 0.78) / cols as f32);
                let p = |s: f32, d: f32| lerp(lerp(a0, a1, s), lerp(b0, b1, s), d);
                if let Some(q) = mq_poly(&[p(s0, d0), p(s1, d0), p(s1, d1), p(s0, d1)]) { c.fill(&q, Cv::solid(col(tone, 0.9))); }
            }
        }
        // the corner of each tower catches the light
        c.line(a0.0, a0.1, b0.0, b0.1, Cv::solid(col(b.paper, 0.55)), u * 0.003);
    }
    // loose pages, falling with him
    for _ in 0..16 {
        let (x, y, s, a) = (vx + (r.f() - 0.5) * u * 1.1, vy + (r.f() - 0.62) * u * 0.8, u * (0.012 + r.f() * 0.026), r.f() * TAU);
        let pts: Vec<_> = [(-1., -0.7), (1., -0.7), (1., 0.7), (-1., 0.7)].iter().map(|&(px, py): &(f32, f32)| (x + (px * a.cos() - py * a.sin()) * s, y + (px * a.sin() + py * a.cos()) * s)).collect();
        mq_paste(c, b, &pts, if r.f() < 0.2 { b.red } else { b.paper }, 1., 1., &mut r);
    }
    // the man in paper: arms flung up, coat open, one leg kicked out; turned a quarter as he goes
    let (fx, fy, fs, rot) = (vx - u * 0.04, vy - u * 0.12, u * 0.26, 0.42_f32);
    let (ca, sa) = (rot.cos(), rot.sin());
    let m = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> { pts.iter().map(|&(px, py)| (fx + (px * ca - py * sa) * fs, fy + (px * sa + py * ca) * fs)).collect() };
    let limb = |a: (f32, f32), z: (f32, f32), wd: f32| -> Vec<(f32, f32)> {
        let (dx, dy) = (z.0 - a.0, z.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(1e-4);
        let (nx, ny) = (-dy / l * wd, dx / l * wd);
        vec![(a.0 + nx, a.1 + ny), (z.0 + nx * 0.8, z.1 + ny * 0.8), (z.0 - nx * 0.8, z.1 - ny * 0.8), (a.0 - nx, a.1 - ny)]
    };
    let mut parts: Vec<Vec<(f32, f32)>> = vec![
        limb((-0.1, -0.22), (-0.3, -0.4), 0.04), limb((-0.3, -0.4), (-0.36, -0.66), 0.034),
        limb((0.1, -0.22), (0.33, -0.2), 0.04), limb((0.33, -0.2), (0.5, -0.38), 0.034),
        limb((-0.05, 0.1), (-0.12, 0.4), 0.05), limb((-0.12, 0.4), (-0.06, 0.66), 0.042),
        limb((0.05, 0.1), (0.24, 0.32), 0.05), limb((0.24, 0.32), (0.46, 0.34), 0.042),
        vec![(-0.115, -0.27), (0.115, -0.27), (0.085, 0.13), (-0.085, 0.13)],
        vec![(-0.09, 0.0), (-0.24, 0.2), (-0.08, 0.16)], vec![(0.09, 0.0), (0.22, 0.24), (0.08, 0.16)],
        mq_disc(0., -0.37, 0.085, 0.09, 20),
    ];
    for p in parts.iter_mut() { *p = m(p); }
    for p in &parts { mq_paste(c, b, p, b.paper, 1., 2.2, &mut r); }
    mq_paste(c, b, &m(&[(-0.012, -0.26), (0.02, -0.255), (0.15, -0.43), (0.12, -0.46)]), b.red, 1., 0.5, &mut r);
    mq_hat(c, b, fx + u * 0.3, fy - u * 0.16, u * 0.1, -0.7, b.red, b.ink, &mut r);
    // our own ledge at the foot of the sheet, where the title stands
    mq_paste(c, b, &[(-u * 0.02, h * 0.79), (w + u * 0.02, h * 0.76), (w + u * 0.02, h + u * 0.02), (-u * 0.02, h + u * 0.02)], b.ink, 1., 2., &mut r);
    c.line(-u * 0.02, h * 0.79 - u * 0.005, w + u * 0.02, h * 0.76 - u * 0.005, Cv::solid(col(b.red, 1.)), u * 0.006);
    mq_titles(c, b, "ACT  IV", "THE FALL", w * 0.055, h * 0.955, u * 0.11, b.red, b.paper);
    mq_billing(c, b, w * 0.5, h * 0.9, u * 0.56, b.paper, 0.8);
}

/// Finale: the last reel. Red curtains, the screen saying so, the beam over the heads, and the hat in row C
fn mq_end(c: &mut Cv, b: &Bill) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(605);
    mq_ground(c, b.ink, 605);
    let (sw, sh) = (w * 0.44, w * 0.44 * 0.42);
    let (sx, sy) = (w / 2. - sw / 2., h * 0.2);
    // the stage under the screen, dimly lit
    mq_paste(c, b, &mq_box(-u * 0.02, sy + sh + u * 0.03, w + u * 0.04, u * 0.14), b.red.mix(BLACK, 0.62), 1., 0., &mut r);
    // the screen, with a soft spill of its light around it
    c.cloud(w / 2., sy + sh / 2., sw * 0.85, b.paper, 0.1);
    mq_paste(c, b, &mq_box(sx, sy, sw, sh), b.paper, 1., 1., &mut r);
    mq_set(c, "THE END", &b.disp, sh * 0.42, w / 2., sy + sh * 0.6, 0.5, sw * 0.7, b.ink, 1.);
    mq_set(c, "GOOD NIGHT", &b.mono, sh * 0.06, w / 2., sy + sh * 0.8, 0.5, sw * 0.5, b.ink, 0.75);
    // the beam from the booth behind us, dust in it
    if let Some(p) = mq_poly(&[(w * 0.48, -h * 0.05), (w * 0.52, -h * 0.05), (sx + sw, sy), (sx + sw, sy + sh), (sx, sy + sh), (sx, sy)]) {
        c.fill(&p, Cv::linear(0., 0., 0., sy + sh, &[(0., col(b.paper, 0.0)), (1., col(b.paper, 0.08))]));
        c.clip_path(&p);
        for _ in 0..140 { c.circle(w * 0.5 + (r.f() - 0.5) * sw, r.f() * (sy + sh), u * 0.0012 * (0.5 + r.f()), Cv::solid(col(b.paper, 0.2 + r.f() * 0.4))); }
        c.clip = None;
    }
    // the curtains: deep folds cut in two reds, gathered at the hem, and the valance scalloped across the top
    for side in [-1_f32, 1.] {
        let (edge, outer) = (if side < 0. { sx + u * 0.02 } else { sx + sw - u * 0.02 }, if side < 0. { -u * 0.02 } else { w + u * 0.02 });
        let folds = 7;
        for k in 0..folds {
            let (f0, f1) = (k as f32 / folds as f32, (k + 1) as f32 / folds as f32);
            let x0 = outer + (edge - outer) * f0;
            let x1 = outer + (edge - outer) * f1;
            let bot = h * 0.74 + (r.f() - 0.5) * u * 0.02;
            let tone = if k % 2 == 0 { b.red } else { b.red.mix(BLACK, 0.28) };
            mq_paste(c, b, &[(x0, -u * 0.02), (x1, -u * 0.02), (x1 + (x0 - x1) * 0.1, bot), (x0, bot + u * 0.012)], tone, 1., 0.6, &mut r);
        }
    }
    let mut val = vec![(-u * 0.02, -u * 0.02), (w + u * 0.02, -u * 0.02)];
    let n = 14;
    for k in (0..=n * 4).rev() {
        let t = k as f32 / (n * 4) as f32;
        let x = -u * 0.02 + (w + u * 0.04) * t;
        let s = ((t * n as f32).fract() * PI).sin();
        val.push((x, h * 0.105 + s * u * 0.035));
    }
    mq_paste(c, b, &val, b.red.mix(BLACK, 0.12), 1., 1.6, &mut r);
    c.line(0., h * 0.1, w, h * 0.1, Cv::solid(col(b.paper, 0.8)), u * 0.003);
    mq_set(c, "FINALE", &b.body, u * 0.03, w / 2., h * 0.087, 0.5, w, b.paper, 0.95);
    // the house: three rows of seats, a few heads against the screen's light; one of them still wears the hat
    for (row, y) in [(0_usize, h * 0.74), (1, h * 0.84), (2, h * 0.95)] {
        let sw2 = u * (0.075 + row as f32 * 0.012);
        let mut x = -sw2 * (0.3 + 0.4 * row as f32);
        let mut i = 0;
        while x < w {
            let head = (row == 0 && (i == 6 || i == 13 || i == 17)) || (row == 1 && (i == 3 || i == 11)) || (row == 2 && i == 8);
            if head {
                let (hx, hr) = (x + sw2 * 0.5, sw2 * 0.24);
                c.circle(hx, y - sw2 * 0.55, hr, Cv::solid(col(b.ink, 1.)));
                c.fill_rect(hx - hr * 1.5, y - sw2 * 0.4, hr * 3., sw2 * 0.5, Cv::solid(col(b.ink, 1.)));
                if row == 0 && i == 13 { mq_hat(c, b, hx, y - sw2 * 0.7, hr * 3.4, 0.0, b.ink, b.red, &mut r); }
            }
            let back = [(x + sw2 * 0.06, y - sw2 * 0.3), (x + sw2 * 0.94, y - sw2 * 0.3), (x + sw2 * 0.98, y + sw2 * 0.6), (x + sw2 * 0.02, y + sw2 * 0.6)];
            mq_paste(c, b, &back, b.ink.mix(b.red, 0.1), 1., 0., &mut r);
            c.line(back[0].0, back[0].1, back[1].0, back[1].1, Cv::solid(col(b.red, 0.75 - row as f32 * 0.18)), u * 0.003);
            x += sw2 * 1.04;
            i += 1;
        }
    }
    mq_billing(c, b, w / 2., sy + sh + u * 0.075, u * 0.6, b.paper, 0.75);
}

fn mq_finish(c: &mut Cv, b: &Bill, seed: u32) {
    let (w, h, u) = (c.w, c.h, b.u);
    let mut r = Rng::new(seed as u64 * 7 + 3);
    // screen-print dropouts where the ink did not take, and specks where it did not belong
    for _ in 0..(w * h / 2600.) as usize { let (x, y) = (r.f() * w, r.f() * h); c.circle(x, y, u * (0.0006 + r.f() * 0.0012), Cv::solid(col(b.paper, 0.08 + r.f() * 0.22))); }
    for _ in 0..(w * h / 9000.) as usize { let (x, y) = (r.f() * w, r.f() * h); c.circle(x, y, u * (0.0005 + r.f() * 0.001), Cv::solid(col(b.ink, 0.12 + r.f() * 0.2))); }
    // folded in three for the post: two creases across, pale on the ridge and a soft shadow under it
    let lw = (u * 0.0022).max(1.5);
    for f in [1. / 3., 2. / 3.] {
        c.line(0., h * f, w, h * f, Cv::solid(col(b.paper, 0.06)), lw);
        c.line(0., h * f + lw, w, h * f + lw, Cv::solid(col(BLACK, 0.1)), lw * 0.6);
    }
    c.line(w * 0.5, 0., w * 0.5, h, Cv::solid(col(b.paper, 0.045)), lw);
    c.vignette(BLACK, 0.24);
    c.noise(0.05, b.paper, seed);
    c.noise(0.05, BLACK, seed ^ 0x3c1);
}

fn marquee(c: &mut Cv, t: &Theme, i: usize) {
    let b = mq_bill(c, t);
    match i {
        0 => mq_city(c, &b),
        1 => mq_stranger(c, &b),
        2 => mq_road(c, &b),
        3 => mq_fall(c, &b),
        _ => mq_end(c, &b),
    }
    c.reset();
    mq_finish(c, &b, 130 + i as u32);
}

/* ── Dossier: one case, from the first exhibit to the stamp that closes it ────────────────────────────────── */
// A detective's desk under a lamp and the corkboard over it. Manila folders, typed reports with lines blacked out,
// mugshots and index cards pinned up and tied together with red string, red marker rings round what matters.

/// the stationery: paper, manila, marker red, typewriter ink, cork and the desk, and the type
struct Case { u: f32, paper: Rgb, manila: Rgb, red: Rgb, ink: Rgb, cork: Rgb, desk: Rgb, disp: String, body: String, mono: String }
fn ds_case(c: &Cv, t: &Theme) -> Case {
    let p = t.palette;
    let (ink, paper) = if p.ink.luma() <= p.surface.luma() { (p.ink, p.surface) } else { (p.surface, p.ink) };
    Case { u: c.h.min(c.w * 0.5625), paper: paper.mix(WHITE, 0.3), manila: p.bg.mix(Rgb(0xD6, 0xB2, 0x68), 0.55), red: p.accent, ink,
        cork: Rgb(0xA3, 0x76, 0x4A).mix(p.muted, 0.12), desk: Rgb(0x2E, 0x20, 0x16).mix(ink, 0.25),
        disp: t.display.clone(), body: t.body.clone(), mono: t.mono.clone() }
}
const DS_WORDS: [&str; 60] = ["subject", "was", "seen", "leaving", "the", "premises", "at", "approximately", "hours", "witness", "states", "that",
    "a", "dark", "sedan", "no", "plates", "parked", "outside", "on", "the", "night", "of", "the", "13th", "lights", "off", "second", "floor",
    "window", "report", "filed", "by", "officer", "on", "duty", "further", "inquiry", "pending", "see", "attached", "statement", "nothing",
    "taken", "from", "the", "safe", "door", "found", "unlocked", "key", "missing", "rain", "heavy", "until", "midnight", "clerk", "recalls",
    "a", "hat"];

/// the corkboard: thousands of granules, darker and lighter than the board, and the light falling unevenly across it
fn ds_cork(c: &mut Cv, k: &Case, seed: u64) {
    let (w, h, u) = (c.w, c.h, k.u);
    c.fill_all(Cv::solid(col(k.cork, 1.)));
    let mut r = Rng::new(seed);
    for _ in 0..(w * h / 240.) as usize {
        let (x, y, s) = (r.f() * w, r.f() * h, u * (0.0008 + r.f() * 0.0022));
        let tone = if r.f() < 0.55 { k.cork.mix(BLACK, 0.3 + r.f() * 0.3) } else { k.cork.mix(WHITE, 0.15 + r.f() * 0.2) };
        c.circle(x, y, s, Cv::solid(col(tone, 0.45 + r.f() * 0.4)));
    }
    for _ in 0..14 { let (x, y, rad) = (r.f() * w, r.f() * h, (0.2 + r.f() * 0.3) * w); c.cloud(x, y, rad, if r.f() < 0.5 { BLACK } else { WHITE }, 0.05); }
    c.fill_all(Cv::radial(w * 0.45, h * 0.4, w * 0.75, &[(0., col(Rgb(255, 225, 170), 0.1)), (1., col(BLACK, 0.4))]));
}
/// the desk: dark wood, its grain running across, and the pool of the lamp
fn ds_desk(c: &mut Cv, k: &Case, seed: u64) {
    let (w, h, u) = (c.w, c.h, k.u);
    c.fill_all(Cv::solid(col(k.desk, 1.)));
    let mut r = Rng::new(seed);
    for i in 0..110 {
        let y0 = h * i as f32 / 110. + (r.f() - 0.5) * u * 0.012;
        let (amp, fr, ph) = (u * (0.003 + r.f() * 0.01), 0.8 + r.f() * 2.5, r.f() * TAU);
        let mut b = PathBuilder::new();
        for s in 0..=48 { let x = w * s as f32 / 48.; let y = y0 + (x / w * fr * TAU + ph).sin() * amp; if s == 0 { b.move_to(x, y) } else { b.line_to(x, y) } }
        if let Some(p) = b.finish() {
            let tone = if r.f() < 0.6 { BLACK } else { k.desk.mix(WHITE, 0.22) };
            c.stroke(&p, Cv::solid(col(tone, 0.1 + r.f() * 0.12)), u * (0.001 + r.f() * 0.003));
        }
    }
    c.fill_all(Cv::radial(w * 0.55, h * 0.42, w * 0.72, &[(0., col(Rgb(255, 214, 150), 0.18)), (0.55, col(BLACK, 0.1)), (1., col(BLACK, 0.5))]));
}

/// a sheet lying on the board or the desk: a soft shadow, the stock; leaves the canvas set to draw on it (origin at its corner)
fn ds_sheet(c: &mut Cv, k: &Case, cx: f32, cy: f32, w: f32, h: f32, deg: f32, tone: Rgb, lift: f32) -> Transform {
    let tf = Transform::from_translate(cx, cy).pre_rotate(deg).pre_translate(-w / 2., -h / 2.);
    c.tf = Transform::from_translate(k.u * 0.004 * lift, k.u * 0.009 * lift).pre_concat(tf);
    for (g, a) in [(k.u * 0.012, 0.08_f32), (k.u * 0.006, 0.12), (k.u * 0.002, 0.2)] { c.fill_rect(-g, -g, w + 2. * g, h + 2. * g, Cv::solid(col(BLACK, a * lift.min(1.6)))); }
    c.tf = tf;
    c.fill_rect(0., 0., w, h, Cv::solid(col(tone, 1.)));
    c.fill_rect(0., 0., w, h, Cv::linear(0., 0., w, h, &[(0., col(WHITE, 0.1)), (0.6, col(WHITE, 0.)), (1., col(BLACK, 0.08))]));
    tf
}
/// typewritten: each strike a little lighter or heavier, the ribbon bleeding a hair to the right
fn ds_type(c: &mut Cv, s: &str, fam: &str, size: f32, x: f32, y: f32, tone: Rgb, a: f32, r: &mut Rng) -> f32 {
    let mut cx = x;
    let mut buf = [0u8; 4];
    for ch in s.chars() {
        let st = ch.encode_utf8(&mut buf);
        let (kk, dy) = (0.62 + r.f() * 0.38, (r.f() - 0.5) * size * 0.05);
        c.text(st, fam, size, cx + size * 0.03, y + dy, col(tone, a * kk * 0.22));
        cx += c.text(st, fam, size, cx, y + dy, col(tone, a * kk));
    }
    cx - x
}
/// lines of a typed report across a width; now and then a phrase blacked out
fn ds_lines(c: &mut Cv, k: &Case, x: f32, y: f32, wid: f32, n: usize, size: f32, redact: f32, r: &mut Rng) {
    let sp = crate::paint::measure(" ", &k.mono, size);
    for l in 0..n {
        let ly = y + l as f32 * size * 1.55;
        let end = if l + 1 == n { wid * (0.3 + r.f() * 0.4) } else { wid * (0.86 + r.f() * 0.14) };
        let mut cx = x;
        while cx - x < end {
            let wd = DS_WORDS[(r.f() * DS_WORDS.len() as f32) as usize % DS_WORDS.len()];
            let ww = crate::paint::measure(wd, &k.mono, size);
            if cx - x + ww > wid { break; }
            if r.f() < redact {
                let run = ww + r.f() * size * 6.;
                c.fill_rect(cx - size * 0.1, ly - size * 0.78, run.min(x + wid - cx), size * 1.02, Cv::solid(col(BLACK, 0.92)));
                cx += run + sp;
            } else { cx += ds_type(c, wd, &k.mono, size, cx, ly, k.ink, 0.88, r) + sp; }
        }
    }
}
/// a ring in red marker, drawn fast: a loop and a bit more, not quite closing on itself
fn ds_ring(c: &mut Cv, k: &Case, cx: f32, cy: f32, rx: f32, ry: f32, r: &mut Rng) {
    let (turns, a0, tilt) = (1.14 + r.f() * 0.1, r.f() * TAU, (r.f() - 0.5) * 0.4);
    let mut b = PathBuilder::new();
    for i in 0..=90 {
        let t = i as f32 / 90.;
        let a = a0 + t * turns * TAU;
        let s = 1. + (t * 7. + a0).sin() * 0.035 + t * 0.07;
        let (ex, ey) = (a.cos() * rx * s, a.sin() * ry * s);
        let (x, y) = (cx + ex * tilt.cos() - ey * tilt.sin(), cy + ex * tilt.sin() + ey * tilt.cos());
        if i == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
    }
    if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(k.red, 0.3)), k.u * 0.01); c.stroke(&p, Cv::solid(col(k.red, 0.85)), k.u * 0.0055); }
}
/// a stroke of red marker along some points
fn ds_mark(c: &mut Cv, k: &Case, pts: &[(f32, f32)], wd: f32) {
    let mut b = PathBuilder::new();
    for (i, &(x, y)) in pts.iter().enumerate() { if i == 0 { b.move_to(x, y) } else { b.line_to(x, y) } }
    if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(k.red, 0.3)), wd * 1.8); c.stroke(&p, Cv::solid(col(k.red, 0.85)), wd); }
}
/// a paper clip: one wire bent into three nested loops, its shadow under it
fn ds_clip(c: &mut Cv, k: &Case, x: f32, y: f32, len: f32, deg: f32) {
    let tf0 = c.tf;
    let mut b = PathBuilder::new();
    let arc = |b: &mut PathBuilder, cx: f32, cy: f32, rad: f32, a0: f32, a1: f32| { for i in 0..=14 { let a = a0 + (a1 - a0) * i as f32 / 14.; b.line_to(cx + a.cos() * rad, cy + a.sin() * rad); } };
    b.move_to(0.16, 0.7); b.line_to(0.16, 0.22); arc(&mut b, 0.27, 0.22, 0.11, PI, TAU); b.line_to(0.38, 0.86);
    arc(&mut b, 0.22, 0.86, 0.16, 0., PI); b.line_to(0.06, 0.13); arc(&mut b, 0.24, 0.13, 0.18, PI, TAU); b.line_to(0.42, 0.46);
    if let Some(p) = b.finish() {
        let at = |dx: f32, dy: f32| tf0.pre_concat(Transform::from_translate(x + dx, y + dy).pre_rotate(deg).pre_scale(len, len));
        let lw = 0.022;
        c.tf = at(k.u * 0.003, k.u * 0.005); c.stroke(&p, Cv::solid(col(BLACK, 0.35)), lw);
        c.tf = at(0., 0.); c.stroke(&p, Cv::solid(col(Rgb(0x8E, 0x93, 0x99), 1.)), lw);
        c.stroke(&p, Cv::solid(col(WHITE, 0.45)), lw * 0.35);
    }
    c.tf = tf0;
}
/// a push-pin, its head catching the lamp
fn ds_pin(c: &mut Cv, k: &Case, x: f32, y: f32, tone: Rgb) {
    let s = k.u * 0.011;
    c.circle(x + s * 0.45, y + s * 0.7, s * 1.05, Cv::solid(col(BLACK, 0.35)));
    c.circle(x, y, s, Cv::radial(x - s * 0.35, y - s * 0.35, s * 1.3, &[(0., col(tone.mix(WHITE, 0.5), 1.)), (0.55, col(tone, 1.)), (1., col(tone.mix(BLACK, 0.45), 1.))]));
}
/// red string pulled from pin to pin, sagging a little, with its shadow on the board
fn ds_string(c: &mut Cv, k: &Case, a: (f32, f32), z: (f32, f32), sag: f32) {
    let mid = ((a.0 + z.0) / 2., (a.1 + z.1) / 2. + sag);
    for (dx, dy, tone, al, wd) in [(k.u * 0.004, k.u * 0.007, BLACK, 0.3, 0.0028), (0., 0., k.red, 0.95, 0.0026)] {
        let mut b = PathBuilder::new();
        b.move_to(a.0 + dx, a.1 + dy); b.quad_to(mid.0 + dx, mid.1 + dy, z.0 + dx, z.1 + dy);
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(tone, al)), k.u * wd); }
    }
}
/// a rubber stamp: a double border and the word, with flecks where the rubber did not take
fn ds_stamp(c: &mut Cv, k: &Case, s: &str, cx: f32, cy: f32, size: f32, deg: f32, under: Rgb, r: &mut Rng) {
    let tf0 = c.tf;
    let (w, h) = (crate::paint::measure(s, &k.disp, size) + size * 0.8, size * 1.35);
    c.tf = tf0.pre_concat(Transform::from_translate(cx, cy).pre_rotate(deg));
    let ink = col(k.red, 0.8);
    for (g, lw) in [(0., size * 0.08), (size * 0.15, size * 0.035)] {
        if let Some(rc) = tiny_skia::Rect::from_xywh(-w / 2. + g, -h / 2. + g, w - 2. * g, h - 2. * g) { c.stroke(&PathBuilder::from_rect(rc), Cv::solid(ink), lw); }
    }
    c.text(s, &k.disp, size, -w / 2. + size * 0.4, size * 0.36, ink);
    for _ in 0..((w * h) / (size * size) * 26.) as usize {
        let (x, y) = (-w / 2. + r.f() * w, -h / 2. + r.f() * h);
        c.circle(x, y, size * (0.01 + r.f() * 0.035), Cv::solid(col(under, 0.55 + r.f() * 0.4)));
    }
    c.tf = tf0;
}
/// a fingerprint: a whorl of broken ridges, flatter towards the crease
fn ds_print(c: &mut Cv, k: &Case, cx: f32, cy: f32, rx: f32, ry: f32, deg: f32, r: &mut Rng) {
    let tf0 = c.tf;
    c.tf = tf0.pre_concat(Transform::from_translate(cx, cy).pre_rotate(deg));
    let rings = 20;
    for i in 1..=rings {
        let f = i as f32 / rings as f32;
        let (ox, oy) = (rx * 0.1 * (1. - f), -ry * 0.14 * (1. - f));
        let (g0, gap) = (r.f() * TAU, 0.15 + r.f() * 0.5);
        let mut b = PathBuilder::new();
        let mut pen = false;
        for s in 0..=96 {
            let a = s as f32 / 96. * TAU;
            if (a - g0).rem_euclid(TAU) < gap { pen = false; continue; }
            let flat = if a.sin() > 0. { 1. - 0.3 * f * a.sin() } else { 1. };
            let wob = 1. + (a * 5. + i as f32).sin() * 0.018;
            let (x, y) = (ox + a.cos() * rx * f * wob, oy + a.sin() * ry * f * wob * flat);
            if pen { b.line_to(x, y) } else { b.move_to(x, y); pen = true; }
        }
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(k.ink, 0.5 + r.f() * 0.3)), rx * 0.024); }
    }
    c.tf = tf0;
}
/// a photograph with a white border; `scene` paints the picture inside it (origin at the picture's corner)
fn ds_photo(c: &mut Cv, k: &Case, cx: f32, cy: f32, w: f32, h: f32, deg: f32, lift: f32, seed: u64, scene: &dyn Fn(&mut Cv, f32, f32, &mut Rng)) -> Transform {
    let tf = ds_sheet(c, k, cx, cy, w, h, deg, WHITE.mix(k.paper, 0.4), lift);
    let m = w * 0.06;
    let (iw, ih) = (w - 2. * m, h - m * 3.2);
    if let Some(rc) = tiny_skia::Rect::from_xywh(m, m, iw, ih) { c.clip_path(&PathBuilder::from_rect(rc)); }
    c.tf = tf.pre_translate(m, m);
    let mut r = Rng::new(seed);
    scene(c, iw, ih, &mut r);
    c.fill_rect(0., 0., iw, ih, Cv::linear(0., 0., iw, ih, &[(0., col(WHITE, 0.12)), (0.4, col(WHITE, 0.)), (1., col(BLACK, 0.15))]));
    c.clip = None;
    c.tf = tf;
    tf
}
/// a mugshot: the height chart behind, a head and shoulders in silhouette, the board under the chin
fn ds_mug(c: &mut Cv, k: &Case, w: f32, h: f32, r: &mut Rng, no: &str, hat: bool) {
    let (lite, dark) = (k.paper.mix(k.ink, 0.25), k.ink.mix(k.paper, 0.12));
    c.fill_rect(0., 0., w, h, Cv::linear(0., 0., 0., h, &[(0., col(lite, 1.)), (1., col(lite.mix(k.ink, 0.2), 1.))]));
    for i in 1..9 {
        let y = h * i as f32 / 9.;
        c.line(0., y, w, y, Cv::solid(col(k.ink, 0.35)), h * 0.006);
        c.text(&format!("{}", 7 - i as i32 / 2), &k.mono, h * 0.05, w * 0.03, y - h * 0.01, col(k.ink, 0.5));
    }
    let (hx, hy, hr) = (w * (0.5 + (r.f() - 0.5) * 0.06), h * 0.44, w * 0.16);
    if let Some(p) = mq_poly(&[(hx - w * 0.36, h), (hx - w * 0.3, h * 0.74), (hx - w * 0.08, h * 0.66), (hx + w * 0.08, h * 0.66), (hx + w * 0.3, h * 0.74), (hx + w * 0.36, h)]) { c.fill(&p, Cv::solid(col(dark, 1.))); }
    c.fill_rect(hx - hr * 0.42, hy + hr * 0.6, hr * 0.84, hr * 0.8, Cv::solid(col(dark, 1.)));
    if let Some(p) = Cv::ellipse(hx, hy, hr, hr * 1.22, 0.) { c.fill(&p, Cv::solid(col(dark, 1.))); }
    if hat {
        if let Some(p) = mq_poly(&[(hx - hr * 1.7, hy - hr * 0.62), (hx + hr * 1.7, hy - hr * 0.55), (hx + hr * 1.5, hy - hr * 0.4), (hx - hr * 1.5, hy - hr * 0.45)]) { c.fill(&p, Cv::solid(col(dark, 1.))); }
        if let Some(p) = mq_poly(&[(hx - hr * 0.95, hy - hr * 0.6), (hx - hr * 0.85, hy - hr * 1.45), (hx, hy - hr * 1.62), (hx + hr * 0.85, hy - hr * 1.45), (hx + hr * 0.95, hy - hr * 0.6)]) { c.fill(&p, Cv::solid(col(dark, 1.))); }
    }
    let (bw, bh) = (w * 0.46, h * 0.13);
    c.fill_rect(hx - bw / 2., h * 0.8, bw, bh, Cv::solid(col(BLACK, 1.)));
    mq_set(c, no, &k.mono, bh * 0.55, hx, h * 0.8 + bh * 0.7, 0.5, bw * 0.9, WHITE, 0.9);
}
/// an index card: white, a red rule along the top, faint blue-grey ruling, and its typed lines
fn ds_card(c: &mut Cv, k: &Case, cx: f32, cy: f32, w: f32, h: f32, deg: f32, lines: &[&str], r: &mut Rng) -> Transform {
    let tf = ds_sheet(c, k, cx, cy, w, h, deg, WHITE.mix(k.paper, 0.25), 0.8);
    c.fill_rect(0., h * 0.2, w, h * 0.012, Cv::solid(col(k.red, 0.6)));
    let mut y = h * 0.2 + h * 0.17;
    while y < h * 0.96 { c.fill_rect(0., y, w, h * 0.006, Cv::solid(col(Rgb(0x6E, 0x8C, 0xA8), 0.35))); y += h * 0.17; }
    for (i, l) in lines.iter().enumerate() {
        let size = if i == 0 { h * 0.13 } else { h * 0.1 };
        let yy = if i == 0 { h * 0.16 } else { h * 0.2 + h * 0.17 * i as f32 - h * 0.025 };
        ds_type(c, l, if i == 0 { &k.disp } else { &k.mono }, size, w * 0.07, yy, k.ink, 0.92, r);
    }
    tf
}
/// a manila folder lying open: the back leaf, the tab, and the front leaf folded back under
fn ds_folder(c: &mut Cv, k: &Case, cx: f32, cy: f32, w: f32, h: f32, deg: f32, tab: &str, tab_x: f32, r: &mut Rng) -> Transform {
    let tf = Transform::from_translate(cx, cy).pre_rotate(deg).pre_translate(-w / 2., -h / 2.);
    let tab_w = w * 0.3;
    let outline = [(0., h * 0.06), (tab_x * w, h * 0.06), (tab_x * w + h * 0.02, 0.), (tab_x * w + tab_w, 0.), (tab_x * w + tab_w + h * 0.02, h * 0.06), (w, h * 0.06), (w, h), (0., h)];
    c.tf = Transform::from_translate(k.u * 0.006, k.u * 0.012).pre_concat(tf);
    if let Some(p) = mq_poly(&outline) { c.fill(&p, Cv::solid(col(BLACK, 0.35))); }
    c.tf = tf;
    if let Some(p) = mq_poly(&outline) {
        c.fill(&p, Cv::solid(col(k.manila, 1.)));
        c.fill(&p, Cv::linear(0., 0., w, h, &[(0., col(WHITE, 0.12)), (1., col(BLACK, 0.12))]));
        c.stroke(&p, Cv::solid(col(k.manila.mix(BLACK, 0.3), 0.6)), k.u * 0.0015);
    }
    // the score lines where it folds, and the tab's typed label
    for f in [0.985_f32, 0.97] { c.line(0., h * f, w, h * f, Cv::solid(col(k.manila.mix(BLACK, 0.25), 0.4)), k.u * 0.0015); }
    c.fill_rect(tab_x * w + h * 0.03, h * 0.012, tab_w - h * 0.04, h * 0.042, Cv::solid(col(WHITE.mix(k.paper, 0.3), 0.95)));
    ds_type(c, tab, &k.mono, h * 0.027, tab_x * w + h * 0.045, h * 0.044, k.ink, 0.9, r);
    tf
}
/// an evidence bag: the thing inside drawn first, then the plastic, the red strip and the white label
fn ds_bag(c: &mut Cv, k: &Case, cx: f32, cy: f32, w: f32, h: f32, deg: f32, item: &str, r: &mut Rng, inside: &dyn Fn(&mut Cv, f32, f32)) {
    let tf = Transform::from_translate(cx, cy).pre_rotate(deg).pre_translate(-w / 2., -h / 2.);
    c.tf = Transform::from_translate(k.u * 0.003, k.u * 0.006).pre_concat(tf);
    c.fill_rect(0., 0., w, h, Cv::solid(col(BLACK, 0.18)));
    c.tf = tf;
    inside(c, w, h);
    c.fill_rect(0., 0., w, h, Cv::solid(col(WHITE, 0.13)));
    c.fill_rect(0., 0., w, h, Cv::linear(0., 0., w, h, &[(0., col(WHITE, 0.2)), (0.35, col(WHITE, 0.)), (0.6, col(WHITE, 0.12)), (1., col(WHITE, 0.))]));
    if let Some(rc) = tiny_skia::Rect::from_xywh(0., 0., w, h) { c.stroke(&PathBuilder::from_rect(rc), Cv::solid(col(WHITE, 0.4)), k.u * 0.002); }
    c.fill_rect(0., 0., w, h * 0.1, Cv::solid(col(k.red, 0.92)));
    mq_set(c, "EVIDENCE", &k.body, h * 0.06, w / 2., h * 0.075, 0.5, w * 0.8, WHITE, 0.95);
    c.fill_rect(w * 0.08, h * 0.74, w * 0.84, h * 0.2, Cv::solid(col(WHITE.mix(k.paper, 0.3), 0.95)));
    ds_type(c, item, &k.mono, h * 0.05, w * 0.12, h * 0.82, k.ink, 0.9, r);
    ds_type(c, "CASE 1105-B", &k.mono, h * 0.042, w * 0.12, h * 0.9, k.ink, 0.7, r);
}
/// a numbered evidence tent standing on the desk
fn ds_tent(c: &mut Cv, k: &Case, x: f32, y: f32, s: f32, n: &str) {
    if let Some(p) = mq_poly(&[(x - s * 0.5, y), (x + s * 0.5, y), (x + s * 0.42, y - s * 0.7), (x - s * 0.42, y - s * 0.7)]) {
        c.tf = Transform::from_translate(s * 0.12, s * 0.12);
        c.fill(&p, Cv::solid(col(BLACK, 0.35)));
        c.tf = Transform::identity();
        c.fill(&p, Cv::solid(col(Rgb(0xE8, 0xC2, 0x3A).mix(k.manila, 0.25), 1.)));
    }
    c.fill_rect(x - s * 0.42, y - s * 0.72, s * 0.84, s * 0.08, Cv::solid(col(BLACK, 0.85)));
    mq_set(c, n, &k.body, s * 0.46, x, y - s * 0.12, 0.5, s, BLACK, 0.9);
}
/// a key, in brass
fn ds_key(c: &mut Cv, x: f32, y: f32, s: f32, deg: f32) {
    let tf0 = c.tf;
    c.tf = tf0.pre_concat(Transform::from_translate(x, y).pre_rotate(deg));
    let brass = Rgb(0xB8, 0x93, 0x4C);
    if let Some(p) = Cv::arc(0., 0., s * 0.22, s * 0.22, 0., TAU) { c.stroke(&p, Cv::solid(col(brass, 1.)), s * 0.09); }
    c.fill_rect(s * 0.2, -s * 0.045, s * 0.75, s * 0.09, Cv::solid(col(brass, 1.)));
    for (dx, l) in [(0.62_f32, 0.12_f32), (0.74, 0.08), (0.86, 0.14)] { c.fill_rect(s * dx, s * 0.04, s * 0.07, s * l, Cv::solid(col(brass, 1.))); }
    c.fill_rect(s * 0.2, -s * 0.045, s * 0.75, s * 0.02, Cv::solid(col(WHITE, 0.35)));
    c.tf = tf0;
}

/// Evidence: the open folder under the lamp: the fingerprint card, the photograph, the key in its bag, the tents
fn ds_evidence(c: &mut Cv, k: &Case) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(701);
    ds_desk(c, k, 701);
    ds_folder(c, k, w * 0.56, h * 0.55, u * 1.22, u * 0.78, -3., "CASE 1105-B  EXHIBITS", 0.08, &mut r);
    c.reset();
    // the fingerprint card
    let tf = ds_sheet(c, k, w * 0.43, h * 0.5, u * 0.42, u * 0.54, -6., k.paper, 1.);
    ds_type(c, "FINGERPRINT CARD", &k.disp, u * 0.026, u * 0.03, u * 0.05, k.ink, 0.9, &mut r);
    ds_type(c, "LATENT · LIFTED FROM DOOR HANDLE", &k.mono, u * 0.012, u * 0.03, u * 0.075, k.ink, 0.75, &mut r);
    for (i, lbl) in ["R. THUMB", "R. INDEX", "R. MIDDLE"].iter().enumerate() {
        let bx = u * 0.03 + i as f32 * u * 0.123;
        if let Some(rc) = tiny_skia::Rect::from_xywh(bx, u * 0.1, u * 0.115, u * 0.15) { c.stroke(&PathBuilder::from_rect(rc), Cv::solid(col(k.ink, 0.6)), u * 0.0015); }
        ds_type(c, lbl, &k.mono, u * 0.0095, bx + u * 0.006, u * 0.115, k.ink, 0.7, &mut r);
        ds_print(c, k, bx + u * 0.058, u * 0.185, u * 0.034, u * 0.046, (r.f() - 0.5) * 20., &mut r);
    }
    if let Some(rc) = tiny_skia::Rect::from_xywh(u * 0.03, u * 0.27, u * 0.36, u * 0.24) { c.stroke(&PathBuilder::from_rect(rc), Cv::solid(col(k.ink, 0.6)), u * 0.0015); }
    ds_type(c, "LATENT No. 4  (ENLARGED)", &k.mono, u * 0.0105, u * 0.04, u * 0.29, k.ink, 0.75, &mut r);
    ds_print(c, k, u * 0.21, u * 0.4, u * 0.075, u * 0.1, 8., &mut r);
    ds_ring(c, k, u * 0.24, u * 0.38, u * 0.05, u * 0.04, &mut r);
    c.tf = tf;
    ds_mark(c, k, &[(u * 0.3, u * 0.35), (u * 0.37, u * 0.31)], u * 0.004);
    c.reset();
    // the photograph of the door, clipped to the folder
    ds_photo(c, k, w * 0.66, h * 0.38, u * 0.3, u * 0.36, 5., 1.2, 33, &|c, pw, ph, _r| {
        let (dk, lt) = (k.ink.mix(k.paper, 0.1), k.paper.mix(k.ink, 0.35));
        c.fill_rect(0., 0., pw, ph, Cv::solid(col(dk, 1.)));
        c.fill_rect(pw * 0.3, ph * 0.18, pw * 0.4, ph * 0.82, Cv::solid(col(lt, 1.)));
        c.fill_rect(pw * 0.34, ph * 0.24, pw * 0.32, ph * 0.76, Cv::solid(col(dk.mix(lt, 0.3), 1.)));
        c.circle(pw * 0.61, ph * 0.62, pw * 0.02, Cv::solid(col(lt, 1.)));
        c.fill_rect(pw * 0.27, ph * 0.12, pw * 0.46, ph * 0.06, Cv::solid(col(lt.mix(dk, 0.3), 1.)));
        mq_set(c, "13", &k.body, ph * 0.08, pw * 0.5, ph * 0.17, 0.5, pw, dk, 1.);
        c.fill_all(Cv::radial(pw * 0.5, ph * 0.1, pw * 0.9, &[(0., col(WHITE, 0.2)), (1., col(BLACK, 0.4))]));
    });
    ds_type(c, "REAR DOOR, 23:40", &k.mono, u * 0.016, u * 0.03, u * 0.345, k.ink, 0.85, &mut r);
    c.reset();
    ds_clip(c, k, w * 0.66 - u * 0.1, h * 0.38 - u * 0.215, u * 0.1, 8.);
    // the key, bagged
    ds_bag(c, k, w * 0.78, h * 0.66, u * 0.24, u * 0.32, -12., "ITEM 04 · KEY, BRASS", &mut r, &|c, bw, bh| ds_key(c, bw * 0.24, bh * 0.42, bw * 0.62, -30.));
    c.reset();
    ds_ring(c, k, w * 0.765, h * 0.625, u * 0.1, u * 0.075, &mut r);
    // tents on the desk
    for (x, y, n) in [(w * 0.16, h * 0.86, "1"), (w * 0.9, h * 0.33, "2"), (w * 0.92, h * 0.9, "3")] { ds_tent(c, k, x, y, u * 0.07, n); }
}

/// Suspects: the corkboard. Five faces, their cards, red string to the one in the middle; one is ringed
fn ds_suspects(c: &mut Cv, k: &Case) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(702);
    ds_cork(c, k, 702);
    // a newspaper cutting at the left
    let tf = ds_sheet(c, k, w * 0.14, h * 0.62, u * 0.36, u * 0.5, -4., k.paper.mix(k.manila, 0.35), 0.7);
    mq_set(c, "THE EVENING STAR", &k.disp, u * 0.022, u * 0.18, u * 0.062, 0.5, u * 0.32, k.ink, 0.9);
    c.fill_rect(u * 0.02, u * 0.072, u * 0.32, u * 0.002, Cv::solid(col(k.ink, 0.8)));
    mq_set(c, "MAN SEEN", &k.body, u * 0.05, u * 0.02, u * 0.125, 0., u * 0.32, k.ink, 0.95);
    mq_set(c, "LEAVING HOTEL", &k.body, u * 0.05, u * 0.02, u * 0.175, 0., u * 0.32, k.ink, 0.95);
    for col_x in [0.02_f32, 0.18] { ds_lines(c, k, u * col_x, u * 0.215, u * 0.14, 16, u * 0.0085, 0., &mut r); }
    c.tf = tf;
    ds_pin(c, k, u * 0.18, u * 0.02, k.red);
    c.reset();
    // the faces
    let spots = [(0.36_f32, 0.3_f32, -3.), (0.56, 0.24, 2.), (0.78, 0.3, -2.), (0.44, 0.66, 3.), (0.7, 0.68, -4.)];
    let names = [("UNKNOWN MALE", "seen 22:15"), ("THE DRIVER", "no plates"), ("NIGHT CLERK", "alibi: none"), ("LANDLADY", "heard a door"), ("THE STRANGER", "hat · coat · ?")];
    let mut pins = vec![];
    for (i, (&(fx, fy, deg), &(name, note))) in spots.iter().zip(names.iter()).enumerate() {
        let (cx, cy, pw, ph) = (w * fx, h * fy, u * 0.19, u * 0.24);
        ds_photo(c, k, cx, cy, pw, ph, deg, 1., 40 + i as u64, &|c, iw, ih, r| ds_mug(c, k, iw, ih, r, &format!("1105-{}", i + 1), i == 4));
        c.reset();
        ds_card(c, k, cx + u * 0.02, cy + ph * 0.66, u * 0.2, u * 0.1, -deg * 0.6, &[name, note], &mut r);
        c.reset();
        pins.push((cx, cy - ph * 0.44));
    }
    // the middle of it all: a typed card with only a question
    let (qx, qy) = (w * 0.58, h * 0.5);
    let tf = ds_card(c, k, qx, qy, u * 0.24, u * 0.12, 2., &["WHO HAD THE KEY", "room 13 · rear door"], &mut r);
    c.tf = tf;
    mq_set(c, "?", &k.body, u * 0.075, u * 0.2, u * 0.108, 0.5, u, k.red, 0.9);
    c.reset();
    let qp = (qx + u * 0.085, qy - u * 0.05);
    for &p in &pins { ds_string(c, k, p, qp, u * 0.03); }
    for &(x, y) in &pins { ds_pin(c, k, x, y, k.red); }
    ds_pin(c, k, qp.0, qp.1, Rgb(0x2C, 0x5A, 0x9A));
    // and the one that matters, ringed twice
    let (sx, sy) = (w * spots[4].0, h * spots[4].1);
    ds_ring(c, k, sx, sy, u * 0.14, u * 0.17, &mut r);
    ds_mark(c, k, &[(sx + u * 0.32, sy + u * 0.1), (sx + u * 0.17, sy + u * 0.03)], u * 0.004);
    ds_mark(c, k, &[(sx + u * 0.2, sy + u * 0.005), (sx + u * 0.17, sy + u * 0.03), (sx + u * 0.205, sy + u * 0.06)], u * 0.004);
}

/// Timeline: the street map pinned up, the route in red marker, and the night told in index cards across the top
fn ds_timeline(c: &mut Cv, k: &Case) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(703);
    ds_cork(c, k, 703);
    let (mw, mh) = (u * 1.2, u * 0.66);
    let (mx, my) = (w * 0.5, h * 0.6);
    let tf = ds_sheet(c, k, mx, my, mw, mh, -1.5, k.paper.mix(k.manila, 0.2), 0.9);
    // the map: blocks between streets, a river with two bridges, a park; streets named in small type
    let block = k.paper.mix(k.manila, 0.45);
    let (cols, rows) = (12, 7);
    for i in 0..cols { for j in 0..rows {
        let (bx, by) = (mw * (i as f32 + 0.12) / cols as f32, mh * (j as f32 + 0.14) / rows as f32);
        if (i == 7 || i == 8) && j < 4 { continue; }
        c.fill_rect(bx, by, mw / cols as f32 * 0.76, mh / rows as f32 * 0.72, Cv::solid(col(if (i, j) == (2, 5) || (i, j) == (3, 5) { Rgb(0x8F, 0xA8, 0x7A) } else { block }, 1.)));
    } }
    let mut riv = PathBuilder::new();
    riv.move_to(-mw * 0.02, mh * 0.62); riv.cubic_to(mw * 0.25, mh * 0.5, mw * 0.4, mh * 0.82, mw * 0.62, mh * 0.66); riv.cubic_to(mw * 0.8, mh * 0.54, mw * 0.9, mh * 0.7, mw * 1.02, mh * 0.6);
    if let Some(p) = riv.finish() { c.stroke(&p, Cv::solid(col(Rgb(0x7D, 0x9C, 0xB4), 0.85)), mh * 0.06); }
    for bx in [0.3_f32, 0.72] { c.fill_rect(mw * bx, mh * 0.5, mw * 0.03, mh * 0.3, Cv::solid(col(block.mix(k.ink, 0.2), 1.))); }
    for (s, x, y, rot) in [("HARBOUR ST", 0.03_f32, 0.105_f32, 0.), ("KING AVE", 0.355, 0.48, 0.), ("CANAL", 0.5, 0.72, -8.)] {
        let tf2 = c.tf; c.tf = tf2.pre_concat(Transform::from_translate(mw * x, mh * y).pre_rotate(rot));
        ds_type(c, s, &k.mono, mh * 0.022, 0., 0., k.ink, 0.6, &mut r);
        c.tf = tf2;
    }
    mq_set(c, "DISTRICT 7", &k.disp, mh * 0.05, mw * 0.97, mh * 0.95, 1., mw, k.ink, 0.75);
    // the route in red marker, dashed where nobody saw him
    let stops = [(0.12_f32, 0.3_f32), (0.35, 0.42), (0.58, 0.32), (0.82, 0.78)];
    for (i, pair) in stops.windows(2).enumerate() {
        let (a, z) = ((mw * pair[0].0, mh * pair[0].1), (mw * pair[1].0, mh * pair[1].1));
        let n = if i == 2 { 9 } else { 1 };
        for s in 0..n {
            if n > 1 && s % 2 == 1 { continue; }
            let (t0, t1) = (s as f32 / n as f32, (s as f32 + 1.) / n as f32);
            ds_mark(c, k, &[(a.0 + (z.0 - a.0) * t0, a.1 + (z.1 - a.1) * t0), (a.0 + (z.0 - a.0) * t1, a.1 + (z.1 - a.1) * t1)], u * 0.004);
        }
    }
    let pin_pts: Vec<(f32, f32)> = stops.iter().map(|&(x, y)| { let mut q = tiny_skia::Point::from_xy(mw * x, mh * y); tf.map_point(&mut q); (q.x, q.y) }).collect();
    let (ex, ey) = (mw * 0.82, mh * 0.78);
    c.tf = tf;
    ds_mark(c, k, &[(ex - u * 0.025, ey - u * 0.025), (ex + u * 0.025, ey + u * 0.025)], u * 0.005);
    ds_mark(c, k, &[(ex + u * 0.025, ey - u * 0.025), (ex - u * 0.025, ey + u * 0.025)], u * 0.005);
    c.reset();
    // the night, card by card, tied down to the map
    let times = [("21:40", "HOTEL · checks in"), ("22:15", "BRIDGE · seen"), ("23:02", "GARAGE · car gone"), ("23:40", "REAR DOOR · ?")];
    for (i, (&(tm, what), &pp)) in times.iter().zip(pin_pts.iter()).enumerate() {
        let (cx, cy) = (w * (0.17 + i as f32 * 0.22), h * 0.16);
        ds_string(c, k, (cx, cy + u * 0.05), pp, u * 0.04);
        ds_card(c, k, cx, cy, u * 0.24, u * 0.12, (r.f() - 0.5) * 6., &[tm, what], &mut r);
        c.reset();
        ds_pin(c, k, cx + u * 0.07, cy - u * 0.045, Rgb(0x2C, 0x5A, 0x9A));
    }
    for &(x, y) in &pin_pts { ds_pin(c, k, x, y, k.red); }
}

/// Reports: the typed pages, fanned out; lines blacked out, a coffee ring, the CONFIDENTIAL stamp and the marker
fn ds_reports(c: &mut Cv, k: &Case) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(704);
    ds_desk(c, k, 704);
    let (pw, ph) = (u * 0.62, u * 0.82);
    for (dx, dy, deg) in [(0.1_f32, 0.03_f32, 7.), (-0.06, 0.02, -5.)] {
        let tf = ds_sheet(c, k, w * 0.5 + u * dx, h * 0.54 + u * dy, pw, ph, deg, k.paper.mix(k.manila, 0.15), 0.8);
        c.tf = tf;
        ds_lines(c, k, pw * 0.1, ph * 0.2, pw * 0.8, 22, u * 0.0125, 0.06, &mut r);
        c.reset();
    }
    let tf = ds_sheet(c, k, w * 0.5, h * 0.55, pw, ph, -1., k.paper, 1.2);
    let fs = u * 0.0135;
    mq_set(c, "INCIDENT REPORT", &k.disp, u * 0.04, pw * 0.5, ph * 0.08, 0.5, pw * 0.8, k.ink, 0.9);
    c.fill_rect(pw * 0.08, ph * 0.1, pw * 0.84, u * 0.002, Cv::solid(col(k.ink, 0.8)));
    for (i, (a, b)) in [("CASE No.", "1105-B"), ("DATE", "13 NOVEMBER"), ("OFFICER", "BADGE 0451"), ("STATUS", "OPEN")].iter().enumerate() {
        let (x, y) = (pw * (0.08 + (i % 2) as f32 * 0.44), ph * (0.145 + (i / 2) as f32 * 0.035));
        ds_type(c, a, &k.mono, fs * 0.85, x, y, k.ink, 0.6, &mut r);
        ds_type(c, b, &k.mono, fs, x + pw * 0.16, y, k.ink, 0.95, &mut r);
    }
    ds_lines(c, k, pw * 0.08, ph * 0.25, pw * 0.84, 26, fs, 0.09, &mut r);
    // the coffee ring
    for (rr, a) in [(u * 0.075, 0.28_f32), (u * 0.071, 0.12)] { if let Some(p) = Cv::arc(pw * 0.78, ph * 0.7, rr, rr * 0.96, 0.4, TAU + 0.1) { c.stroke(&p, Cv::solid(col(Rgb(0x6B, 0x45, 0x22), a)), u * 0.005); } }
    ds_stamp(c, k, "CONFIDENTIAL", pw * 0.55, ph * 0.86, u * 0.045, -12., k.paper, &mut r);
    ds_ring(c, k, pw * 0.42, ph * 0.505, u * 0.11, u * 0.025, &mut r);
    c.reset();
    let _ = tf;
    ds_clip(c, k, w * 0.5 - pw * 0.38, h * 0.55 - ph * 0.52, u * 0.12, -4.);
    // the red marker, uncapped, lying across the corner of the desk
    let tf2 = Transform::from_translate(w * 0.83, h * 0.72).pre_rotate(-35.);
    c.tf = Transform::from_translate(u * 0.006, u * 0.012).pre_concat(tf2);
    c.fill_rect(-u * 0.16, -u * 0.014, u * 0.32, u * 0.028, Cv::solid(col(BLACK, 0.35)));
    c.tf = tf2;
    c.fill_rect(-u * 0.16, -u * 0.014, u * 0.25, u * 0.028, Cv::solid(col(k.red, 1.)));
    c.fill_rect(-u * 0.16, -u * 0.014, u * 0.25, u * 0.007, Cv::solid(col(WHITE, 0.25)));
    c.fill_rect(u * 0.09, -u * 0.012, u * 0.04, u * 0.024, Cv::solid(col(Rgb(0xE8, 0xE4, 0xDA), 1.)));
    if let Some(p) = mq_poly(&[(u * 0.13, -u * 0.009), (u * 0.165, -u * 0.003), (u * 0.165, u * 0.003), (u * 0.13, u * 0.009)]) { c.fill(&p, Cv::solid(col(k.red.mix(BLACK, 0.3), 1.))); }
    mq_set(c, "PERMANENT", &k.body, u * 0.012, -u * 0.02, u * 0.005, 0.5, u, WHITE, 0.8);
    c.reset();
    // its cap, rolled away
    c.circle(w * 0.93 + u * 0.004, h * 0.86 + u * 0.008, u * 0.017, Cv::solid(col(BLACK, 0.35)));
    c.circle(w * 0.93, h * 0.86, u * 0.017, Cv::radial(w * 0.925, h * 0.855, u * 0.02, &[(0., col(k.red.mix(WHITE, 0.3), 1.)), (1., col(k.red.mix(BLACK, 0.3), 1.))]));
}

/// Closed: the folder shut and tied, its cover typed up, CLOSED stamped across it; a photo's corner still showing
fn ds_closed(c: &mut Cv, k: &Case) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(705);
    ds_desk(c, k, 705);
    // the photograph that would not stay inside
    ds_photo(c, k, w * 0.5 + u * 0.66, h * 0.36, u * 0.26, u * 0.3, 14., 0.8, 91, &|c, pw, ph, r| ds_mug(c, k, pw, ph, r, "1105-5", true));
    c.reset();
    let (fw, fh) = (u * 1.12, u * 0.78);
    let tf = ds_folder(c, k, w * 0.5, h * 0.56, fw, fh, 2., "1105-B  ·  CLOSED", 0.62, &mut r);
    c.tf = tf;
    // the cover: the department's block, ruled lines filled in by typewriter
    mq_set(c, "DEPARTMENT OF INVESTIGATION", &k.body, fh * 0.035, fw * 0.08, fh * 0.17, 0., fw, k.ink, 0.75);
    mq_set(c, "CASE FILE", &k.disp, fh * 0.09, fw * 0.08, fh * 0.28, 0., fw, k.ink, 0.9);
    for (i, (a, b)) in [("CASE No.", "1105-B"), ("SUBJECT", "THE STRANGER"), ("OPENED", "12 NOVEMBER"), ("CLOSED", "13 NOVEMBER"), ("FILED BY", "YOU")].iter().enumerate() {
        let y = fh * (0.4 + i as f32 * 0.075);
        mq_set(c, a, &k.body, fh * 0.028, fw * 0.08, y, 0., fw, k.ink, 0.6);
        c.fill_rect(fw * 0.22, y + fh * 0.008, fw * 0.4, u * 0.0015, Cv::solid(col(k.ink, 0.5)));
        ds_type(c, b, &k.mono, fh * 0.032, fw * 0.24, y - fh * 0.004, k.ink, 0.92, &mut r);
    }
    // the string tie: two buttons, the string wound round them in a figure of eight
    let (b1, b2) = ((fw * 0.86, fh * 0.48), (fw * 0.86, fh * 0.62));
    for &(bx, by) in &[b1, b2] {
        c.circle(bx + u * 0.003, by + u * 0.005, fh * 0.035, Cv::solid(col(BLACK, 0.3)));
        c.circle(bx, by, fh * 0.035, Cv::solid(col(k.manila.mix(Rgb(0x7A, 0x52, 0x2C), 0.6), 1.)));
        c.circle(bx, by, fh * 0.012, Cv::solid(col(k.manila.mix(BLACK, 0.5), 1.)));
    }
    let mut b = PathBuilder::new();
    b.move_to(b1.0 - fh * 0.03, b1.1); b.cubic_to(b1.0 - fh * 0.06, b1.1 + fh * 0.1, b2.0 + fh * 0.06, b2.1 - fh * 0.1, b2.0 + fh * 0.03, b2.1);
    b.move_to(b1.0 + fh * 0.03, b1.1); b.cubic_to(b1.0 + fh * 0.06, b1.1 + fh * 0.1, b2.0 - fh * 0.06, b2.1 - fh * 0.1, b2.0 - fh * 0.03, b2.1);
    b.move_to(b2.0, b2.1 + fh * 0.03); b.cubic_to(b2.0 + fh * 0.02, b2.1 + fh * 0.12, b2.0 - fh * 0.08, b2.1 + fh * 0.2, b2.0 - fh * 0.04, fh * 1.02);
    if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(Rgb(0xE8, 0xDF, 0xC8), 0.95)), u * 0.0035); }
    ds_stamp(c, k, "CLOSED", fw * 0.6, fh * 0.76, fh * 0.15, -14., k.manila, &mut r);
    ds_mark(c, k, &[(fw * 0.31, fh * 0.69), (fw * 0.325, fh * 0.715), (fw * 0.37, fh * 0.65)], u * 0.004);
    c.reset();
}

fn ds_finish(c: &mut Cv, k: &Case, seed: u32) {
    c.vignette(BLACK, 0.3);
    c.noise(0.05, k.paper, seed);
    c.noise(0.06, BLACK, seed ^ 0x77);
}

fn dossier(c: &mut Cv, t: &Theme, i: usize) {
    let k = ds_case(c, t);
    match i {
        0 => ds_evidence(c, &k),
        1 => ds_suspects(c, &k),
        2 => ds_timeline(c, &k),
        3 => ds_reports(c, &k),
        _ => ds_closed(c, &k),
    }
    c.reset();
    ds_finish(c, &k, 150 + i as u32);
}

/* ── Xerox: a punk zine, five issues, photocopied one time too many ───────────────────────────────────────── */
// Library copier paper, toner that never quite took, halftone photographs cut out with scissors, headlines made of
// letters cut from other magazines, parcel tape, and one cyan, overprinted a little out of register.

/// the copy shop: paper, toner, the one cyan, and the type
struct Zine { u: f32, paper: Rgb, ink: Rgb, cyan: Rgb, disp: String, body: String, mono: String }
fn xr_zine(c: &Cv, t: &Theme) -> Zine {
    let p = t.palette;
    let (ink, paper) = if p.ink.luma() <= p.bg.luma() { (p.ink, p.bg) } else { (p.bg, p.ink) };
    Zine { u: c.h.min(c.w * 0.5625), paper, ink, cyan: p.accent, disp: t.display.clone(), body: t.body.clone(), mono: t.mono.clone() }
}
/// the faces the headline letters are cut from
const XR_CUTS: [&str; 10] = ["Rubik Dirt", "Alfa Slab One", "Anton", "Playfair Display", "Space Mono", "League Gothic", "Bebas Neue", "Cinzel", "Fraunces", "Archivo Narrow"];

/// copier paper, and the copier itself in it: the lid left open (a black edge), a grey haze where the toner drifted
fn xr_paper(c: &mut Cv, k: &Zine, seed: u64, lid: bool) {
    let (w, h, u) = (c.w, c.h, k.u);
    c.fill_all(Cv::solid(col(k.paper, 1.)));
    let mut r = Rng::new(seed);
    for _ in 0..7 { let (x, y) = (r.f() * w, r.f() * h); c.cloud(x, y, (0.15 + r.f() * 0.3) * w, k.ink, 0.035); }
    if lid {
        // the lid not shut: the edge of the glass comes out black, ragged where the light got in
        let ew = u * 0.05;
        c.fill_rect(w - ew, 0., ew, h, Cv::linear(w - ew, 0., w, 0., &[(0., col(k.ink, 0.)), (0.45, col(k.ink, 0.75)), (1., col(k.ink, 0.95))]));
        for _ in 0..(h / 3.) as usize { let y = r.f() * h; c.circle(w - ew * (0.3 + r.f() * 0.9), y, u * (0.001 + r.f() * 0.003), Cv::solid(col(k.ink, 0.5 + r.f() * 0.4))); }
    }
}
/// a halftone: dots on a screen turned 45°, each as big as the picture is dark there
fn xr_dots(c: &mut Cv, x0: f32, y0: f32, w: f32, h: f32, cell: f32, deg: f32, ink: Rgb, a: f32, tone: &dyn Fn(f32, f32) -> f32) {
    let (s, co) = deg.to_radians().sin_cos();
    let (cx, cy) = (x0 + w / 2., y0 + h / 2.);
    let n = ((w * w + h * h).sqrt() / cell / 2.) as i32 + 2;
    let mut b = PathBuilder::new();
    for i in -n..=n { for j in -n..=n {
        let (gx, gy) = (i as f32 * cell, j as f32 * cell);
        let (x, y) = (cx + gx * co - gy * s, cy + gx * s + gy * co);
        if x < x0 - cell || x > x0 + w + cell || y < y0 - cell || y > y0 + h + cell { continue; }
        let t = tone(x, y).clamp(0., 1.);
        if t < 0.04 { continue; }
        b.push_circle(x, y, cell * 0.5 * 1.2 * t.sqrt());
    } }
    if let Some(p) = b.finish() { c.fill(&p, Cv::solid(col(ink, a))); }
}
/// shading on a lit ball: 0 where the light falls full on it, 1 in its shadow (light from the upper left)
fn xr_ball(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> Option<f32> {
    let (nx, ny) = ((x - cx) / rx, (y - cy) / ry);
    let d2 = nx * nx + ny * ny;
    if d2 >= 1. { return None; }
    let nz = (1. - d2).sqrt();
    Some(1. - (-0.55 * nx - 0.45 * ny + 0.7 * nz).max(0.))
}
/// how far a point is from a segment (for arms, stems, the spikes of hair)
fn xr_seg(x: f32, y: f32, a: (f32, f32), z: (f32, f32)) -> f32 {
    let (dx, dy) = (z.0 - a.0, z.1 - a.1);
    let t = (((x - a.0) * dx + (y - a.1) * dy) / (dx * dx + dy * dy).max(1e-4)).clamp(0., 1.);
    ((x - a.0 - dx * t).powi(2) + (y - a.1 - dy * t).powi(2)).sqrt()
}
/// the face on the cover: spiked hair, dark glasses, a leather collar; None off the figure. `s` is the head's height
fn xr_face(x: f32, y: f32, cx: f32, cy: f32, s: f32) -> Option<f32> {
    // the glasses: two lenses and the bridge, black, a glint of window across each
    for side in [-1_f32, 1.] {
        let (lx, ly) = (cx + side * 0.155 * s, cy - 0.03 * s);
        let (dx, dy) = ((x - lx) / (0.125 * s), (y - ly) / (0.07 * s));
        if dx.abs().powi(4) + dy.abs().powi(4) < 1. {
            let g = ((x - lx) + (y - ly) * 1.6) / s;
            return Some(if (g + 0.03).abs() < 0.012 { 0.25 } else { 0.97 });
        }
    }
    if (y - (cy - 0.045 * s)).abs() < 0.012 * s && (x - cx).abs() < 0.04 * s { return Some(0.95); }
    // the hair: a cap over the crown, and spikes off it
    const LEN: [f32; 9] = [0.24, 0.4, 0.31, 0.5, 0.36, 0.46, 0.29, 0.38, 0.22];
    for (k, len) in LEN.iter().enumerate() {
        let f = k as f32 / 8.;
        let base = (cx + (f - 0.5) * 0.7 * s, cy - 0.27 * s);
        let tip = (base.0 + (f - 0.5) * 0.62 * s + 0.03 * s, base.1 - len * s);
        let d = xr_seg(x, y, base, tip);
        let t = ((y - tip.1) / (base.1 - tip.1)).clamp(0., 1.);
        if d < 0.06 * s * t + 0.004 * s { return Some(0.9 - 0.3 * (1. - t) * ((cx - x) / s + 0.2).max(0.)); }
    }
    let (hx, hy) = ((x - cx) / (0.4 * s), (y - (cy - 0.2 * s)) / (0.36 * s));
    if hx * hx + hy * hy < 1. && y < cy - 0.17 * s + (x - cx).abs() * 0.15 { return Some(0.92); }
    // the head, the ear, the nose's shadow and the set mouth
    if let Some(t) = xr_ball(x, y, cx, cy, 0.36 * s, 0.48 * s) {
        let mut t = 0.08 + 0.75 * t;
        if (y - (cy + 0.27 * s)).abs() < 0.012 * s && (x - cx).abs() < 0.1 * s { t = 0.9; }
        if x > cx + 0.01 * s && x < cx + 0.06 * s && y > cy + 0.04 * s && y < cy + 0.17 * s { t += 0.3; }
        if y > cy + 0.3 * s { t += 0.15 * ((y - cy - 0.3 * s) / (0.18 * s)); }
        return Some(t);
    }
    if let Some(t) = xr_ball(x, y, cx + 0.37 * s, cy + 0.02 * s, 0.06 * s, 0.11 * s) { return Some(0.35 + 0.5 * t); }
    // the neck in shadow under the jaw, then the jacket with its collar turned up
    if (x - cx).abs() < 0.13 * s && y > cy + 0.3 * s && y < cy + 0.8 * s { return Some(0.55 + 0.35 * ((x - cx) / (0.13 * s)).max(0.) + 0.2 * (1. - (y - cy - 0.3 * s) / (0.5 * s)).max(0.)); }
    let (jx, jy) = ((x - cx) / (1.0 * s), (y - (cy + 1.35 * s)) / (0.72 * s));
    if jx * jx + jy * jy < 1. {
        let collar = (y - (cy + 0.62 * s)).abs() < 0.1 * s && (x - cx).abs() > 0.14 * s && (x - cx).abs() < 0.5 * s;
        return Some(if collar { 0.6 } else { 0.8 + 0.18 * jx - 0.25 * (-jx).max(0.) * (1. - jy.abs()) });
    }
    None
}

/// letters cut out of other magazines and glued in a row: each its own scrap, face, size, colour and tilt
fn xr_ransom(c: &mut Cv, k: &Zine, s: &str, x: f32, y: f32, size: f32, r: &mut Rng) -> f32 {
    let mut cx = x;
    let schemes = [(WHITE.mix(k.paper, 0.3), k.ink), (k.ink, k.paper), (k.cyan, k.ink), (Rgb(0xD8, 0xCF, 0xB0), k.ink), (WHITE, k.ink), (k.paper.mix(k.ink, 0.12), k.ink)];
    let tf0 = c.tf;
    for ch in s.chars() {
        if ch == ' ' { cx += size * 0.34; continue; }
        let fam = XR_CUTS[(r.f() * XR_CUTS.len() as f32) as usize % XR_CUTS.len()];
        let sz = size * (0.82 + r.f() * 0.36);
        let st = ch.to_string();
        let gw = crate::paint::measure(&st, fam, sz);
        let (pad, top) = (sz * 0.12, sz * 0.86);
        let (bw, bh) = (gw + pad * 2., sz * 1.08);
        let (bg, fg) = schemes[(r.f() * schemes.len() as f32) as usize % schemes.len()];
        let (deg, dy) = ((r.f() - 0.5) * 14., (r.f() - 0.5) * size * 0.12);
        c.tf = tf0.pre_concat(Transform::from_translate(cx + bw / 2., y + dy - bh / 2. + sz * 0.1).pre_rotate(deg).pre_translate(-bw / 2., -bh / 2.));
        if let Some(p) = mq_cut(&mq_box(0., 0., bw, bh), k.u * 0.003, r) {
            if let Some(sh) = p.clone().transform(Transform::from_translate(k.u * 0.003, k.u * 0.005)) { c.fill(&sh, Cv::solid(col(BLACK, 0.3))); }
            c.fill(&p, Cv::solid(col(bg, 1.)));
        }
        c.text(&st, fam, sz, pad, top, col(fg, 1.));
        cx += bw * 0.94 + size * 0.03;
    }
    c.tf = tf0;
    cx - x
}
/// a strip of parcel tape, torn at both ends
fn xr_tape(c: &mut Cv, k: &Zine, x: f32, y: f32, w: f32, deg: f32, r: &mut Rng) {
    let tf0 = c.tf;
    let h = k.u * 0.045;
    c.tf = tf0.pre_concat(Transform::from_translate(x, y).pre_rotate(deg));
    let mut pts = vec![];
    for i in 0..=6 { pts.push((-w / 2. + if i % 2 == 0 { 0. } else { h * 0.12 }, -h / 2. + h * i as f32 / 6.)); }
    for i in 0..=6 { pts.push((w / 2. - if i % 2 == 0 { 0. } else { h * 0.12 }, h / 2. - h * i as f32 / 6.)); }
    if let Some(p) = mq_cut(&pts, k.u * 0.0015, r) {
        c.fill(&p, Cv::solid(col(Rgb(0xE8, 0xE2, 0xC8), 0.55)));
        c.fill(&p, Cv::linear(0., -h / 2., 0., h / 2., &[(0., col(WHITE, 0.25)), (0.4, col(WHITE, 0.)), (1., col(BLACK, 0.06))]));
    }
    c.tf = tf0;
}
/// an embossed label-maker strip: black tape, the letters pressed out of it in white
fn xr_label(c: &mut Cv, k: &Zine, s: &str, x: f32, y: f32, size: f32, deg: f32) {
    let tf0 = c.tf;
    let w = crate::paint::measure(s, &k.mono, size) + size * 1.2;
    c.tf = tf0.pre_concat(Transform::from_translate(x, y).pre_rotate(deg));
    c.fill_rect(size * 0.08, size * 0.14 - size * 0.7, w, size * 1.4, Cv::solid(col(BLACK, 0.3)));
    c.fill_rect(0., -size * 0.7, w, size * 1.4, Cv::solid(col(Rgb(0x16, 0x16, 0x18), 1.)));
    c.fill_rect(0., -size * 0.7, w, size * 0.18, Cv::solid(col(WHITE, 0.08)));
    c.text(s, &k.mono, size, size * 0.6 + size * 0.04, size * 0.38 + size * 0.05, col(BLACK, 0.8));
    c.text(s, &k.mono, size, size * 0.6, size * 0.38, col(WHITE.mix(k.paper, 0.2), 0.95));
    c.tf = tf0;
}
/// typed lines, the ribbon a little dry
fn xr_typed(c: &mut Cv, k: &Zine, lines: &[&str], x: f32, y: f32, size: f32, r: &mut Rng) {
    for (i, l) in lines.iter().enumerate() {
        let mut cx = x;
        for ch in l.chars() {
            let st = ch.to_string();
            cx += c.text(&st, &k.mono, size, cx, y + i as f32 * size * 1.5, col(k.ink, 0.62 + r.f() * 0.35));
        }
    }
}
/// a stamped issue number in a ring
fn xr_issue(c: &mut Cv, k: &Zine, n: &str, x: f32, y: f32, rad: f32, deg: f32) {
    let tf0 = c.tf;
    c.tf = tf0.pre_concat(Transform::from_translate(x, y).pre_rotate(deg));
    for (rr, wd) in [(rad, rad * 0.07), (rad * 0.82, rad * 0.03)] { if let Some(p) = Cv::arc(0., 0., rr, rr, 0., TAU) { c.stroke(&p, Cv::solid(col(k.ink, 0.88)), wd); } }
    mq_set(c, "No.", &k.mono, rad * 0.26, 0., -rad * 0.22, 0.5, rad * 2., k.ink, 0.85);
    mq_set(c, n, &k.disp, rad * 0.78, 0., rad * 0.5, 0.5, rad * 2., k.ink, 0.92);
    c.tf = tf0;
}
/// one cyan, laid over everything in multiply and a little out of register
fn xr_cyan(c: &mut Cv, k: &Zine, path: &tiny_skia::Path, a: f32) {
    let b = c.blend;
    c.blend = BlendMode::Multiply;
    c.fill(path, Cv::solid(col(k.cyan, a)));
    c.blend = b;
}
/// a photograph cut out with scissors: `pts` is the cut, inside it the halftone, its shadow on the page
fn xr_cutout(c: &mut Cv, k: &Zine, x: f32, y: f32, w: f32, h: f32, cell: f32, r: &mut Rng, tone: &dyn Fn(f32, f32) -> f32) {
    if let Some(p) = mq_cut(&mq_box(x, y, w, h), k.u * 0.004, r) {
        if let Some(sh) = p.clone().transform(Transform::from_translate(k.u * 0.004, k.u * 0.007)) { c.fill(&sh, Cv::solid(col(BLACK, 0.22))); }
        c.fill(&p, Cv::solid(col(WHITE.mix(k.paper, 0.5), 1.)));
        c.clip_path(&p);
        xr_dots(c, x, y, w, h, cell, 45., k.ink, 0.95, tone);
        c.clip = None;
    }
}

/// No.1 — Static: the face on the cover, the cyan block behind it, the headline cut out letter by letter
fn xr_static(c: &mut Cv, k: &Zine) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(801);
    xr_paper(c, k, 801, true);
    let (px, py, pw, ph) = (w * 0.47, h * 0.12, w * 0.42, h * 0.82);
    if let Some(p) = mq_poly(&[(px + pw * 0.3, py - h * 0.03), (px + pw * 1.04, py + h * 0.02), (px + pw * 0.98, py + ph * 0.7), (px + pw * 0.24, py + ph * 0.66)]) { xr_cyan(c, k, &p, 0.85); }
    let (fx, fy, fs) = (px + pw * 0.52, py + ph * 0.42, ph * 0.5);
    xr_cutout(c, k, px, py, pw, ph, u * 0.0085, &mut r, &|x, y| xr_face(x, y, fx, fy, fs).unwrap_or(0.4 + 0.25 * ((x - px) / pw) - 0.14 * ((y - py) / ph)));
    xr_tape(c, k, px + pw * 0.08, py + u * 0.01, u * 0.2, -32., &mut r);
    xr_tape(c, k, px + pw * 0.94, py + ph - u * 0.01, u * 0.2, -28., &mut r);
    xr_issue(c, k, "1", w * 0.11, h * 0.2, u * 0.075, -10.);
    xr_ransom(c, k, "STATIC", w * 0.05, h * 0.42, u * 0.12, &mut r);
    xr_typed(c, k, &["issue one.", "photocopied at the library", "after closing. forty copies.", "read it, then pass it on."], w * 0.06, h * 0.55, u * 0.024, &mut r);
    xr_label(c, k, "FREE · PASS IT ON", w * 0.07, h * 0.78, u * 0.028, -3.);
}

/// the cassette: body, label, the two windows with the reels in them, the screws; in its own units (0..1 × 0..0.64)
fn xr_cassette(x: f32, y: f32) -> Option<f32> {
    if x < 0. || x > 1. || y < 0. || y > 0.64 { return None; }
    let edge = x.min(1. - x).min(y).min(0.64 - y);
    let mut t = 0.78 + 0.15 * x - 0.1 * (1. - y / 0.64);
    if edge < 0.012 { t = 0.95; }
    for (sx, sy) in [(0.04, 0.04), (0.96, 0.04), (0.04, 0.6), (0.96, 0.6), (0.5, 0.58)] { let d = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt(); if d < 0.014 { return Some(if d < 0.005 { 0.9 } else { 0.2 }); } }
    // the label, light, ruled
    if x > 0.07 && x < 0.93 && y > 0.06 && y < 0.4 {
        let mut l = 0.08;
        if y < 0.12 { l = 0.75; }
        if (y > 0.16 && y < 0.165) || (y > 0.22 && y < 0.225) { l = 0.6; }
        // the window band across the label, with the two reels
        if y > 0.24 && y < 0.38 && x > 0.2 && x < 0.8 {
            for rx in [0.33_f32, 0.67] {
                let d = ((x - rx).powi(2) + (y - 0.31).powi(2)).sqrt();
                if d < 0.055 {
                    let a = (y - 0.31).atan2(x - rx);
                    let teeth = (a * 6.).sin() > 0.6 && d > 0.028;
                    return Some(if d < 0.022 { 0.15 } else if teeth { 0.85 } else { 0.3 + d * 4. });
                }
            }
            l = if x > 0.42 && x < 0.58 { 0.45 } else { 0.98 };
        }
        return Some(l);
    }
    // the bottom tab, trapezoid
    if y > 0.46 && y < 0.62 && x > 0.18 + (0.62 - y) * 0.6 && x < 0.82 - (0.62 - y) * 0.6 { t = 0.62 + 0.2 * ((x - 0.5).abs() < 0.03) as i32 as f32; }
    Some(t)
}
/// No.2 — Tape: a mixtape blown up past the point of sense, the tracklist typed on a scrap
fn xr_tapes(c: &mut Cv, k: &Zine) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(802);
    xr_paper(c, k, 802, false);
    let (cs, cx, cy, rot) = (u * 1.05, w * 0.6, h * 0.55, -9_f32.to_radians());
    let (sn, cn) = rot.sin_cos();
    let (px, py, pw, ph) = (w * 0.24, h * 0.1, w * 0.73, h * 0.86);
    xr_cutout(c, k, px, py, pw, ph, u * 0.0095, &mut r, &|x, y| {
        let (dx, dy) = (x - cx, y - cy);
        let (lx, ly) = ((dx * cn + dy * sn) / cs + 0.5, (-dx * sn + dy * cn) / cs + 0.32);
        xr_cassette(lx, ly).unwrap_or(0.06 + 0.16 * ((x - px) / pw))
    });
    if let Some(p) = Cv::ellipse(cx - cs * 0.15, cy - cs * 0.01, u * 0.13, u * 0.13, 0.) { xr_cyan(c, k, &p, 0.85); }
    // the label on the tape, written over
    let tf0 = c.tf;
    c.tf = Transform::from_translate(cx, cy).pre_rotate(-9.);
    xr_label(c, k, "NOISE FOR NOBODY", -cs * 0.36, -cs * 0.22, u * 0.03, 1.5);
    c.tf = tf0;
    xr_ransom(c, k, "SIDE A", w * 0.04, h * 0.25, u * 0.11, &mut r);
    // the tracklist on a torn scrap
    let (sx, sy, sw, sh) = (w * 0.04, h * 0.58, u * 0.48, u * 0.3);
    mq_paste(c, &Bill { u, red: k.cyan, ink: k.ink, paper: k.paper, disp: k.disp.clone(), body: k.body.clone(), mono: k.mono.clone() }, &mq_box(sx, sy, sw, sh), WHITE.mix(k.paper, 0.4), 1., 1., &mut r);
    xr_typed(c, k, &["1. noise for nobody", "2. the copier ate it", "3. static (live)", "4. rewind, rewind", "5. side b is blank"], sx + u * 0.03, sy + u * 0.06, u * 0.024, &mut r);
    xr_tape(c, k, sx + sw * 0.5, sy, u * 0.16, 4., &mut r);
    xr_issue(c, k, "2", w * 0.12, h * 0.45, u * 0.06, 8.);
}

/// No.3 — Live: the light from the stage and the hands going up into it; the flyer down the left
fn xr_live(c: &mut Cv, k: &Zine) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(803);
    xr_paper(c, k, 803, true);
    // the crowd: heads and shoulders in rows, arms up
    let mut heads = vec![];
    let mut arms = vec![];
    for row in 0..3 {
        let by = h * (0.8 + row as f32 * 0.07);
        let n = 9 + row * 2;
        for i in 0..n {
            let hx = w * 0.3 + w * 0.7 * (i as f32 + r.f() * 0.6) / n as f32;
            let hr = u * (0.035 + row as f32 * 0.008) * (0.85 + r.f() * 0.3);
            heads.push((hx, by + (r.f() - 0.5) * u * 0.03, hr));
            if r.f() < 0.7 {
                let side = if r.f() < 0.5 { -1. } else { 1. };
                let sh = (hx + side * hr * 1.2, by + hr * 1.4);
                let el = (sh.0 + side * hr * (0.6 + r.f() * 0.8), sh.1 - hr * (2. + r.f()));
                let hd = (el.0 + side * hr * (r.f() - 0.3), el.1 - hr * (1.8 + r.f() * 0.8));
                arms.push((sh, el, hd, hr * 0.34));
            }
        }
    }
    let (lx, ly) = (w * 0.66, h * 0.3);
    let (px, py, pw, ph) = (w * 0.3, h * 0.1, w * 0.68, h * 0.88);
    xr_cutout(c, k, px, py, pw, ph, u * 0.0085, &mut r, &|x, y| {
        for &(hx, hy, hr) in &heads { if ((x - hx) / hr).powi(2) + ((y - hy) / (hr * 1.15)).powi(2) < 1. || (y > hy + hr * 0.8 && (x - hx).abs() < hr * (1.4 + (y - hy - hr) / hr * 0.4)) { return 1.; } }
        for &(a, b, z, rr) in &arms { if xr_seg(x, y, a, b) < rr || xr_seg(x, y, b, z) < rr * 0.85 || ((x - z.0).powi(2) + (y - z.1).powi(2)).sqrt() < rr * 1.3 { return 1.; } }
        let (dx, dy) = (x - lx, y - ly);
        let d = (dx * dx + dy * dy).sqrt() / u;
        let a = dy.atan2(dx);
        let rays = (a * 9.).cos().max(0.).powi(8) * (-d * 1.2).exp() * 0.8;
        let haze = (-((y - h * 0.74) / (u * 0.1)).abs()).exp() * 0.75;
        let glow = (-d * 3.2).exp() + rays + haze;
        0.97 - 0.97 * glow.min(1.)
    });
    // the cyan, a band struck across the flyer
    if let Some(p) = mq_poly(&[(0., h * 0.62), (w * 0.42, h * 0.55), (w * 0.42, h * 0.64), (0., h * 0.71)]) { xr_cyan(c, k, &p, 0.9); }
    xr_ransom(c, k, "LIVE", w * 0.04, h * 0.3, u * 0.15, &mut r);
    xr_ransom(c, k, "TONITE", w * 0.04, h * 0.47, u * 0.085, &mut r);
    xr_label(c, k, "DOORS 8PM", w * 0.05, h * 0.6, u * 0.03, -2.);
    xr_label(c, k, "THE BASEMENT", w * 0.07, h * 0.67, u * 0.03, 1.5);
    xr_typed(c, k, &["three bands. one amp.", "bring a friend, not a camera."], w * 0.05, h * 0.78, u * 0.022, &mut r);
    xr_issue(c, k, "3", w * 0.12, h * 0.9, u * 0.05, 12.);
    xr_tape(c, k, px + pw * 0.5, py + u * 0.004, u * 0.22, 2., &mut r);
}

/// No.4 — Concrete: the estate from across the road, the cyan sun printed where it nearly was
fn xr_concrete(c: &mut Cv, k: &Zine) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(804);
    xr_paper(c, k, 804, false);
    let mut blocks = vec![];
    let mut x = w * 0.03;
    while x < w * 0.98 {
        let bw = u * (0.12 + r.f() * 0.16);
        let top = h * (0.22 + r.f() * 0.38);
        blocks.push((x, bw, top, r.f()));
        x += bw + u * (0.01 + r.f() * 0.04);
    }
    let lit: Vec<u64> = (0..40).map(|_| (r.f() * 4000.) as u64).collect();
    let (sx, sy, sr) = (w * 0.68, h * 0.32, u * 0.13);
    let (px, py, pw, ph) = (w * 0.02, h * 0.1, w * 0.96, h * 0.72);
    xr_cutout(c, k, px, py, pw, ph, u * 0.0085, &mut r, &|x, y| {
        let ground = py + ph * 0.93;
        if y > ground { return 0.85; }
        for &(bx, bw, top, v) in &blocks {
            if v > 0.55 && (x - (bx + bw * 0.3)).abs() < u * 0.003 && y > top - u * 0.07 && y < top { return 0.9; }
            if v > 0.55 && (y - (top - u * 0.05)).abs() < u * 0.003 && (x - (bx + bw * 0.3)).abs() < u * 0.025 { return 0.9; }
            if x >= bx && x <= bx + bw && y >= top {
                let side = x > bx + bw * 0.78;
                let (fx, fy) = ((x - bx) / bw, (y - top) / u);
                // the windows, floor by floor, a few lit
                let (col_i, row_i) = ((fx * 6.) as u64, (fy / 0.042) as u64);
                let inwin = (fx * 6.).fract() > 0.25 && (fx * 6.).fract() < 0.8 && (fy / 0.042).fract() > 0.3 && (fy / 0.042).fract() < 0.8 && !side;
                if inwin { return if lit.contains(&((col_i * 131 + row_i * 7 + (bx as u64)) % 4000)) { 0.05 } else { 0.88 }; }
                return if side { 0.82 } else { 0.48 + v * 0.18 };
            }
        }
        // the sky, darker going up, the sun a pale disc in it
        let d = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
        if d < sr * 0.9 { return 0.04; }
        0.15 + 0.5 * (1. - (y - py) / ph).max(0.) * 0.8
    });
    if let Some(p) = Cv::ellipse(sx + u * 0.03, sy - u * 0.02, sr, sr, 0.) { xr_cyan(c, k, &p, 0.85); }
    xr_ransom(c, k, "CONCRETE", w * 0.05, h * 0.9, u * 0.1, &mut r);
    xr_typed(c, k, &["no.4 — the estate,", "from the fourteenth floor."], w * 0.6, h * 0.88, u * 0.024, &mut r);
    xr_issue(c, k, "4", w * 0.92, h * 0.9, u * 0.055, -6.);
    xr_tape(c, k, w * 0.12, h * 0.1, u * 0.2, -6., &mut r);
    xr_tape(c, k, w * 0.86, h * 0.1, u * 0.2, 5., &mut r);
}

/// No.5 — Copy of a copy: the cover face six times over, each a copy of the last, until it is only toner
fn xr_copies(c: &mut Cv, k: &Zine) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(805);
    xr_paper(c, k, 805, true);
    let (cw, ch, gx, gy) = (w * 0.205, h * 0.34, w * 0.016, h * 0.06);
    let (x0, y0) = (w * 0.31, h * 0.13);
    for g in 0..6 {
        let (x, y) = (x0 + (g % 3) as f32 * (cw + gx), y0 + (g / 3) as f32 * (ch + gy));
        let gf = g as f32;
        let (fx, fy, fs) = (x + cw * (0.5 + gf * 0.01), y + ch * (0.42 + gf * 0.008), ch * 0.5);
        let seed = 900 + g as u64;
        let noise = move |x: f32, y: f32| { let v = ((x * 12.9898 + y * 78.233 + seed as f32).sin() * 43758.545).fract(); v };
        xr_cutout(c, k, x, y, cw, ch, u * (0.0065 + gf * 0.0022), &mut r, &move |x, y| {
            let t = xr_face(x, y, fx, fy, fs).unwrap_or(0.22);
            let t = ((t - 0.5) * (1. + gf * 0.45) + 0.5) + gf * 0.06 + (noise(x, y) - 0.5) * gf * 0.12;
            t
        });
        xr_typed(c, k, &[&format!("GEN {}", g + 1)], x + u * 0.01, y + ch + u * 0.03, u * 0.02, &mut r);
        if g == 5 { if let Some(p) = mq_poly(&mq_box(x - u * 0.01, y + ch * 0.55, cw * 0.7, ch * 0.2)) { xr_cyan(c, k, &p, 0.85); } }
    }
    xr_ransom(c, k, "COPY", w * 0.04, h * 0.32, u * 0.1, &mut r);
    xr_ransom(c, k, "OF A", w * 0.06, h * 0.45, u * 0.09, &mut r);
    xr_ransom(c, k, "COPY", w * 0.04, h * 0.58, u * 0.1, &mut r);
    xr_typed(c, k, &["every copy loses something.", "keep copying."], w * 0.05, h * 0.7, u * 0.022, &mut r);
    xr_issue(c, k, "5", w * 0.1, h * 0.17, u * 0.06, -12.);
    xr_label(c, k, "LAST ISSUE", w * 0.05, h * 0.84, u * 0.028, -2.);
}

/// the copier's last pass over all of it: toner specks, a streak down the drum, the grain of the copy
fn xr_finish(c: &mut Cv, k: &Zine, seed: u32) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(seed as u64 + 77);
    for _ in 0..(w * h / 2600.) as usize {
        let (x, y) = (r.f() * w, r.f() * h);
        c.circle(x, y, u * (0.0004 + r.f() * r.f() * 0.0025), Cv::solid(col(k.ink, 0.25 + r.f() * 0.6)));
    }
    for _ in 0..2 { let x = r.f() * w; c.fill_rect(x, 0., u * (0.001 + r.f() * 0.003), h, Cv::solid(col(k.ink, 0.06 + r.f() * 0.08))); }
    c.fill_rect(0., h * (0.3 + r.f() * 0.4), w, u * 0.012, Cv::solid(col(k.ink, 0.03)));
    c.vignette(k.ink, 0.18);
    c.noise(0.06, k.ink, seed);
}

fn xerox(c: &mut Cv, t: &Theme, i: usize) {
    let k = xr_zine(c, t);
    match i {
        0 => xr_static(c, &k),
        1 => xr_tapes(c, &k),
        2 => xr_live(c, &k),
        3 => xr_concrete(c, &k),
        _ => xr_copies(c, &k),
    }
    c.reset();
    xr_finish(c, &k, 160 + i as u32);
}

/* ── Etch: five copper plates, cut with a burin, printed black, one point of light in each ──────────────────── */
// White-line engraving: the plate is black and every tone is made of cut lines, wider where the picture is lit,
// crossed by a second set where it is brightest. Lines follow the forms they describe. Each plate has one light in
// it (a candle, a lantern, the moon, a stub of candle, the sun) and the plate mark, the title and the engraver's
// signature under the picture, as on an old print.

/// the engraver's bench: the plate, the ink, the one light, the picture's rectangle, the type
struct Plate { u: f32, ink: Rgb, dark: Rgb, glow: Rgb, rect: (f32, f32, f32, f32), disp: String, body: String }
fn et_plate(c: &Cv, t: &Theme) -> Plate {
    let p = t.palette;
    let (ink, dark) = if p.ink.luma() >= p.bg.luma() { (p.ink, p.bg) } else { (p.bg, p.ink) };
    let (w, h) = (c.w, c.h);
    Plate { u: h.min(w * 0.5625), ink, dark, glow: p.accent, rect: (w * 0.05, h * 0.055, w * 0.9, h * 0.74), disp: t.display.clone(), body: t.body.clone() }
}
type Bend<'a> = &'a dyn Fn(f32, f32) -> f32;

/// a soft, slow pattern for clouds, rock and plaster (0..1)
fn et_n(x: f32, y: f32) -> f32 {
    let a = (x * 1.31 + (y * 0.73).sin() * 1.9).sin();
    let b = (y * 1.67 + (x * 0.91).sin() * 1.4).sin();
    let c = ((x + y) * 2.71 + (x * 0.37).cos()).sin();
    (a * 0.45 + b * 0.35 + c * 0.2) * 0.5 + 0.5
}
/// how much a light reaches a point: full at the flame, half at `r`
fn et_lamp(x: f32, y: f32, lx: f32, ly: f32, r: f32) -> f32 { let d = ((x - lx).powi(2) + (y - ly).powi(2)).sqrt() / r; 1. / (1. + d * d) }
/// a round form lit from (lx, ly, lz): 0 turned away, 1 facing the light; None off it
fn et_ball(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32, l: (f32, f32, f32)) -> Option<f32> {
    let (nx, ny) = ((x - cx) / rx, (y - cy) / ry);
    let d2 = nx * nx + ny * ny;
    if d2 >= 1. { return None; }
    let nz = (1. - d2).sqrt();
    let m = (l.0 * l.0 + l.1 * l.1 + l.2 * l.2).sqrt();
    Some(((nx * l.0 + ny * l.1 + nz * l.2) / m).max(0.))
}

/// the burin: parallel lines `gap` apart at `deg` across the picture, each swelling where `tone` is light (between
/// `lo` and `hi`) and pushed sideways by `bend` so it follows the form. Nothing is cut where tone is below `lo`.
#[allow(clippy::too_many_arguments)]
fn et_cut(c: &mut Cv, k: &Plate, gap: f32, deg: f32, lo: f32, hi: f32, scale: f32, bend: Bend, tone: &dyn Fn(f32, f32) -> f32) {
    let (x0, y0, w, h) = k.rect;
    let (s, co) = deg.to_radians().sin_cos();
    let (dx, dy, nx, ny) = (co, s, -s, co);
    let (cx, cy) = (x0 + w / 2., y0 + h / 2.);
    let half = (w * w + h * h).sqrt() / 2. + gap;
    let step = (gap * 0.6).max(1.2);
    let n = (half / gap) as i32 + 1;
    let mut pb = PathBuilder::new();
    let (mut top, mut bot): (Vec<(f32, f32)>, Vec<(f32, f32)>) = (vec![], vec![]);
    fn flush(pb: &mut PathBuilder, top: &mut Vec<(f32, f32)>, bot: &mut Vec<(f32, f32)>) {
        if top.len() >= 2 {
            pb.move_to(top[0].0, top[0].1);
            for p in &top[1..] { pb.line_to(p.0, p.1); }
            for p in bot.iter().rev() { pb.line_to(p.0, p.1); }
            pb.close();
        }
        top.clear();
        bot.clear();
    }
    for j in -n..=n {
        let jo = j as f32 * gap;
        // no two lines cut quite alike
        let wob = (j as f32 * 2.399).sin() * 0.08;
        let mut t = -half;
        while t <= half {
            let (bx, by) = (cx + dx * t + nx * jo, cy + dy * t + ny * jo);
            let mut hw = 0.;
            let (mut px, mut py) = (bx, by);
            if bx >= x0 - gap && bx <= x0 + w + gap && by >= y0 - gap && by <= y0 + h + gap {
                let off = bend(bx, by);
                px = bx + nx * off;
                py = by + ny * off;
                let v = ((tone(px, py) - lo) / (hi - lo)).clamp(0., 1.);
                hw = gap * 0.5 * scale * v * (1. + wob + 0.06 * (t * 0.05 / gap).sin());
            }
            if hw < 0.12 { flush(&mut pb, &mut top, &mut bot); } else { top.push((px + nx * hw, py + ny * hw)); bot.push((px - nx * hw, py - ny * hw)); }
            t += step;
        }
        flush(&mut pb, &mut top, &mut bot);
    }
    if let Some(p) = pb.finish() { c.fill(&p, Cv::solid(col(k.ink, 0.93))); }
}
/// a whole picture: `scene` says which part a point is in and how light it is; each part is cut with its own lines
/// (id, gap, angle, cross-hatch angle, bend). The cross set only comes in where the part is brightest.
fn et_scene(c: &mut Cv, k: &Plate, scene: &dyn Fn(f32, f32) -> (u8, f32), layers: &[(u8, f32, f32, Option<f32>, Bend)]) {
    let (x0, y0, w, h) = k.rect;
    if let Some(p) = mq_poly(&mq_box(x0, y0, w, h)) { c.clip_path(&p); }
    let flat = |_: f32, _: f32| 0_f32;
    for &(id, gap, deg, cross, bend) in layers {
        let tone = |x: f32, y: f32| { let (i, t) = scene(x, y); if i == id { t } else { -1. } };
        et_cut(c, k, gap, deg, 0.03, 0.64, 0.92, bend, &tone);
        if let Some(cd) = cross { et_cut(c, k, gap * 1.1, cd, 0.58, 1.05, 0.75, &flat, &tone); }
    }
    c.clip = None;
}

/// the one light: a glow laid on in screen, a hot core, and the engraver's star of fine rays
fn et_light(c: &mut Cv, k: &Plate, x: f32, y: f32, r: f32, rays: bool) {
    let b = c.blend;
    c.blend = BlendMode::Screen;
    c.cloud(x, y, r * 7., k.glow, 0.16);
    c.cloud(x, y, r * 2.6, k.glow, 0.45);
    c.cloud(x, y, r * 1.1, WHITE, 0.95);
    if rays {
        for i in 0..8 {
            let a = i as f32 * PI / 4. + PI / 8.;
            let len = if i % 2 == 0 { r * 6. } else { r * 3.2 };
            let s = Cv::radial(x, y, len, &[(0., col(WHITE, 0.8)), (1., col(WHITE, 0.))]);
            c.line(x, y, x + a.cos() * len, y + a.sin() * len, s, k.u * if i % 2 == 0 { 0.0018 } else { 0.0011 });
        }
    }
    c.blend = b;
}
/// a candle flame: the teardrop, the blue root, the wick
fn et_flame(c: &mut Cv, k: &Plate, x: f32, y: f32, fh: f32) {
    let fw = fh * 0.3;
    let mut b = PathBuilder::new();
    b.move_to(x, y - fh);
    b.cubic_to(x + fw * 0.4, y - fh * 0.55, x + fw * 1.15, y - fh * 0.1, x, y + fw * 0.6);
    b.cubic_to(x - fw * 1.15, y - fh * 0.1, x - fw * 0.4, y - fh * 0.55, x, y - fh);
    b.close();
    if let Some(p) = b.finish() {
        c.fill(&p, Cv::radial(x, y - fh * 0.2, fh * 0.75, &[(0., col(WHITE, 1.)), (0.55, col(k.glow, 0.95)), (1., col(k.glow, 0.5))]));
    }
    c.circle(x, y + fw * 0.15, fw * 0.4, Cv::solid(col(k.dark.mix(Rgb(0x2A, 0x3A, 0x60), 0.6), 0.55)));
    c.line(x, y + fw * 0.6, x + fw * 0.15, y - fh * 0.12, Cv::solid(col(k.dark, 0.9)), fh * 0.035);
}
/// stars: each a tiny cut cross
fn et_stars(c: &mut Cv, k: &Plate, n: usize, seed: u64, ok: &dyn Fn(f32, f32) -> bool) {
    let mut r = Rng::new(seed);
    let (x0, y0, w, h) = k.rect;
    for _ in 0..n {
        let (x, y) = (x0 + r.f() * w, y0 + r.f() * h);
        if !ok(x, y) { continue; }
        let s = k.u * (0.002 + r.f() * r.f() * 0.006);
        c.line(x - s, y, x + s, y, Cv::solid(col(k.ink, 0.85)), k.u * 0.0012);
        c.line(x, y - s, x, y + s, Cv::solid(col(k.ink, 0.85)), k.u * 0.0012);
    }
}
/// the print round the picture: the plate mark pressed into the paper, the border rules, the title cut in capitals
/// between two fleurons, and the signatures, "inv." under the left corner and "sculp." under the right
fn et_frame(c: &mut Cv, k: &Plate, title: &str, sub: &str) {
    let (x0, y0, w, h) = k.rect;
    let (cw, ch, u) = (c.w, c.h, k.u);
    let m = u * 0.016;
    for (inset, a, wd) in [(m, 0.2, u * 0.0022), (m + u * 0.004, 0.08, u * 0.0012)] {
        if let Some(p) = mq_poly(&mq_box(inset, inset, cw - inset * 2., ch - inset * 2.)) { c.stroke(&p, Cv::solid(col(k.ink, a)), wd); }
    }
    if let Some(p) = mq_poly(&mq_box(x0, y0, w, h)) { c.stroke(&p, Cv::solid(col(k.ink, 0.85)), u * 0.0024); }
    let o = u * 0.007;
    if let Some(p) = mq_poly(&mq_box(x0 - o, y0 - o, w + o * 2., h + o * 2.)) { c.stroke(&p, Cv::solid(col(k.ink, 0.5)), u * 0.001); }
    let by = y0 + h;
    mq_set(c, "R. S. inv.", &k.body, u * 0.021, x0, by + u * 0.032, 0., w * 0.3, k.ink, 0.6);
    mq_set(c, "R. S. sculp.", &k.body, u * 0.021, x0 + w, by + u * 0.032, 1., w * 0.3, k.ink, 0.6);
    let size = mq_set(c, title, &k.disp, u * 0.052, cw / 2., by + u * 0.083, 0.5, w * 0.6, k.ink, 0.95);
    let tw = crate::paint::measure(title, &k.disp, size);
    for side in [-1_f32, 1.] {
        let (fx, fy, s) = (cw / 2. + side * (tw / 2. + u * 0.035), by + u * 0.083 - size * 0.32, u * 0.011);
        if let Some(p) = mq_poly(&[(fx - s * 1.6, fy), (fx, fy - s * 0.7), (fx + s * 1.6, fy), (fx, fy + s * 0.7)]) { c.fill(&p, Cv::solid(col(k.ink, 0.85))); }
        c.line(fx + side * s * 2.2, fy, fx + side * s * 5.5, fy, Cv::solid(col(k.ink, 0.6)), u * 0.0012);
    }
    mq_set(c, sub, &k.body, u * 0.025, cw / 2., by + u * 0.122, 0.5, w * 0.6, k.ink, 0.72);
}

/// Plate I — Lux in tenebris: a candle on a table at night, a book, an inkwell and its quill
fn et_candle(c: &mut Cv, k: &Plate) {
    let (w, h, u) = (c.w, c.h, k.u);
    let g = u * 0.0072;
    let (cx, top_y, cw) = (w * 0.56, h * 0.405, u * 0.034);
    let (lx, ly) = (cx, h * 0.352);
    let lamp = |x: f32, y: f32| et_lamp(x, y, lx, ly, u * 0.3);
    let (tb, tf) = (h * 0.6, h * 0.665);
    let (hx, hy, hrx, hry) = (cx, h * 0.632, u * 0.1, u * 0.02);
    // the book lying flat: front face, the top cover going back, the side toward the candle
    let (bx0, bx1, bft, bb, dpt, off) = (w * 0.23, w * 0.43, h * 0.565, h * 0.637, u * 0.032, u * 0.035);
    // the inkwell and its quill
    let (ix, itop, ibot, irx) = (w * 0.78, h * 0.568, h * 0.642, u * 0.056);
    let quill: Vec<(f32, f32)> = (0..=40).map(|i| {
        let t = i as f32 / 40.;
        (ix - u * 0.008 + (w * 0.71 - ix) * t - (PI * t).sin() * u * 0.035, itop - u * 0.012 + (h * 0.21 - itop) * t)
    }).collect();
    let on_quill = |x: f32, y: f32| -> Option<f32> {
        let (mut best, mut bt) = (f32::MAX, 0.);
        for (i, p) in quill.iter().enumerate() { let d = (x - p.0).powi(2) + (y - p.1).powi(2); if d < best { best = d; bt = i as f32 / 40.; } }
        let d = best.sqrt();
        if d < u * 0.0032 { return Some(0.98); }
        let vane = if bt > 0.22 { u * 0.026 * (PI * ((bt - 0.22) / 0.78).min(1.)).sin().powf(0.6) } else { 0. };
        if d < vane { return Some(0.45 + 0.4 * (1. - d / vane)); }
        None
    };
    let scene = |x: f32, y: f32| -> (u8, f32) {
        let l = lamp(x, y);
        if let Some(t) = on_quill(x, y) { return (6, t * (0.6 + 0.4 * (l * 2.).min(1.))); }
        // the candle: the pool of wax on top, the body turned round like a column, brighter up by the flame
        let ex = ((x - cx) / cw).powi(2) + ((y - top_y) / (cw * 0.3)).powi(2);
        if ex < 1. { return (4, 0.98); }
        if (x - cx).abs() < cw && y > top_y && y < hy {
            let nx = (x - cx) / cw;
            let drip = (x - cx - cw * 0.6).abs() < cw * 0.12 && y < top_y + u * 0.05;
            return (4, (0.2 + 0.62 * (1. - nx * nx).sqrt() + if drip { 0.2 } else { 0. }) * (0.5 + 0.5 * (1. - (y - top_y) / (hy - top_y)).powf(0.6)));
        }
        // the holder: a dish with a lit rim, its foot in shadow
        let e = ((x - hx) / hrx).powi(2) + ((y - hy) / hry).powi(2);
        if e < 1. { return (5, if e > 0.62 { 0.9 } else { 0.42 + 0.45 * l }); }
        if (x - hx).abs() < hrx * 0.55 && y > hy && y < hy + u * 0.014 { return (5, 0.18); }
        // the inkwell: a squat bottle lit from the candle's side, a bright line round its lip
        if (x - ix).abs() < irx && y > itop && y < ibot {
            let nx = (x - ix) / irx;
            if y < itop + u * 0.006 { return (7, 0.85); }
            return (7, (0.08 + 0.7 * (-nx * 0.8 + 0.4).max(0.) * (1. - nx * nx).sqrt()) * (0.6 + l));
        }
        if (x - ix).abs() < irx * 0.45 && y > itop - u * 0.014 && y <= itop { return (7, 0.55 * (1. - (x - ix) / (irx * 0.45))); }
        // the book: the top cover, the side toward the light, and the front with its block of pages
        if y > bft - dpt && y <= bft { let s = (bft - y) / dpt * off; if x > bx0 + s && x < bx1 + s { return (8, 0.95 * l.min(1.) * 1.4); } }
        if x >= bx1 && x < bx1 + off { let s = (x - bx1) / off * dpt; if y > bft - s && y < bb - s { return (9, (0.85 * l * 1.5).min(0.95)); } }
        if x > bx0 && x < bx1 && y > bft && y < bb {
            let f = (y - bft) / (bb - bft);
            return (10, if f < 0.14 || f > 0.86 { 0.3 * l * 1.6 } else { 0.68 * l * 1.6 * (0.7 + 0.3 * x / bx1) });
        }
        // the wall, then the table: its top in the light, the edge, the front falling away into the dark
        if y < tb { return (1, (l * 1.05 * (0.9 + 0.1 * et_n(x / u * 9., y / u * 9.))).min(1.)); }
        if y < tf {
            // the book's shadow thrown left across the table, the inkwell's thrown right
            let mut sh = 1.;
            if x < bx0 && x > bx0 - u * 0.24 && y > tb && y < bb + u * 0.005 { sh = 0.25 + 0.75 * ((bx0 - x) / (u * 0.24)).powi(2); }
            if x > ix + irx * 0.6 && x < ix + irx + u * 0.12 && y > ibot - u * 0.02 && y < ibot + u * 0.008 { sh = 0.3; }
            return (2, (l * 1.15 * sh).min(1.));
        }
        if y < tf + u * 0.006 { return (2, 0.5 * l + 0.3); }
        (3, l * 0.38)
    };
    let flat = |_: f32, _: f32| 0_f32;
    let round = |x: f32, _: f32| -(1. - ((x - cx) / cw).powi(2)).max(0.).sqrt() * u * 0.008;
    let well = |x: f32, _: f32| -(1. - ((x - ix) / irx).powi(2)).max(0.).sqrt() * u * 0.009;
    et_scene(c, k, &scene, &[
        (1, g, 0., Some(65.), &flat), (2, g * 0.85, -3., Some(40.), &flat), (3, g, 90., None, &flat),
        (4, g * 0.75, 0., Some(90.), &round), (5, g * 0.7, 0., None, &flat), (6, g * 0.6, 58., None, &flat),
        (7, g * 0.75, 0., Some(80.), &well), (8, g * 0.8, -12., Some(50.), &flat), (9, g * 0.8, 90., None, &flat), (10, g * 0.55, 0., None, &flat),
    ]);
    et_flame(c, k, lx, ly + u * 0.04, u * 0.075);
    et_light(c, k, lx, ly + u * 0.012, u * 0.02, true);
    et_frame(c, k, "LVX IN TENEBRIS", "Plate I · the candle");
}

/// Plate II — Pharos: a lighthouse on the headland at night, its beam laid across the sky and the sea
fn et_pharos(c: &mut Cv, k: &Plate) {
    let (w, h, u) = (c.w, c.h, k.u);
    let g = u * 0.0072;
    let hz = h * 0.57;
    let (lx, ly) = (w * 0.71, h * 0.266);
    let rock_top = |x: f32| -> f32 {
        if x < w * 0.49 { return hz + (w * 0.49 - x) * 0.55 + (et_n(x / u * 9., 7.) - 0.5) * u * 0.02; }
        let t = (x - w * 0.49) / (w * 0.51);
        let base = hz - (t * 3.).min(1.).powf(0.55) * h * 0.15 + (t - 0.72).max(0.) * h * 0.1;
        base + (et_n(x / u * 14., 3.1) - 0.5) * u * 0.035 * ((x - lx).abs() / (u * 0.07)).min(1.) * t.min(1.)
    };
    let tb = rock_top(lx) + u * 0.012;
    let (tt, ty) = (h * 0.292, h * 0.282);
    let half = |y: f32| u * 0.029 + (y - tt) / (tb - tt) * u * 0.02;
    let (d1, d2) = ((-0.985_f32, -0.17_f32), (0.99_f32, 0.12_f32));
    let beam = |x: f32, y: f32| -> f32 {
        let (vx, vy) = (x - lx, y - ly);
        let mut b: f32 = 0.;
        for (d, reach, a) in [(d1, 1.6, 1.), (d2, 0.35, 0.5)] {
            let along = vx * d.0 + vy * d.1;
            if along <= 0. { continue; }
            let perp = (vx * d.1 - vy * d.0).abs();
            let spread = along * 0.1 + u * 0.012;
            b = b.max(a * (-(perp / spread).powi(2)).exp() * (0.3 + 0.7 * (-along / (u * reach)).exp()));
        }
        b
    };
    let (hx0, hx1, hwall, hroof) = (w * 0.8, w * 0.885, h * 0.392, h * 0.355);
    let scene = |x: f32, y: f32| -> (u8, f32) {
        let rt = rock_top(x);
        // the lantern room is the light itself, cut away and lit later
        if (x - lx).abs() < u * 0.024 && y > h * 0.247 && y < ty { return (0, 0.); }
        if y > h * 0.226 && y <= h * 0.247 && ((x - lx) / (u * 0.027)).powi(2) + ((y - h * 0.247) / (u * 0.021)).powi(2) < 1. {
            return (5, 0.25 + 0.6 * (1. - ((x - lx + u * 0.01) / (u * 0.02)).abs()).max(0.));
        }
        if (x - lx).abs() < u * 0.043 && y >= ty && y < tt { return (5, if y < ty + u * 0.003 { 0.95 } else { 0.5 }); }
        // the tower: banded, round, lit from the beam's side, a door and one lit window
        if y >= tt && y < tb && (x - lx).abs() < half(y) {
            let nx = (x - lx) / half(y);
            let band = (((y - tt) / (tb - tt)) * 4.) as i32 % 2 == 1;
            if (x - lx).abs() < u * 0.006 && (y - h * 0.345).abs() < u * 0.009 { return (4, 1.); }
            if (x - lx).abs() < u * 0.011 && y > tb - u * 0.035 { return (4, 0.02); }
            let t = 0.14 + 0.55 * (1. - (nx + 0.35).abs()).max(0.);
            return (4, if band { t * 0.4 } else { t });
        }
        // the keeper's cottage, its window lit
        if x > hx0 && x < hx1 && y > hwall && y < rt + u * 0.01 {
            if (x - (hx0 + u * 0.03)).abs() < u * 0.008 && (y - (hwall + u * 0.016)).abs() < u * 0.008 { return (6, 1.); }
            return (6, 0.12 + 0.18 * (1. - (x - hx0) / (hx1 - hx0)));
        }
        let mid = (hx0 + hx1) / 2.;
        if y <= hwall && y > hroof + (x - mid).abs() / ((hx1 - hx0) * 0.6) * (hwall - hroof) && (x - mid).abs() < (hx1 - hx0) * 0.6 { return (7, 0.25 + 0.3 * ((mid - x) / (hx1 - hx0)).max(0.)); }
        // the rock: dark broken faces, the top lit by the lantern
        if y >= rt {
            let rim = (-(y - rt) / (u * 0.012)).exp();
            let facet = ((et_n(x / u * 7. + y / u * 4., y / u * 8.) - 0.5) * 3.).clamp(-0.5, 0.5) + 0.5;
            let crack = ((et_n(x / u * 30., y / u * 11.) - 0.5).abs() < 0.012) as i32 as f32;
            let deep = (1. - (y - rt) / (h * 0.45)).max(0.25);
            return (3, (0.05 + 0.22 * facet * deep + 0.35 * rim + 0.45 * et_lamp(x, y, lx, ly, u * 0.14)) * (1. - 0.8 * crack));
        }
        // the sea: dark swell, the beam's light lying on it near the horizon, foam where it breaks on the rock
        if y > hz && y < rt {
            let crest = (x / (u * 0.05) + et_n(x / u * 4., y / u * 8.) * 3. + y / (u * 0.01)).sin() * 0.5 + 0.5;
            let lit = 0.35 * (-(y - hz) / (u * 0.06)).exp() * (x / lx).min(1.);
            let gap = rt - y;
            let foam = if gap < u * 0.025 { 0.55 * et_n(x / u * 40., y / u * 40.) * (1. - gap / (u * 0.025)) } else { 0. };
            return (2, 0.07 + 0.12 * crest + lit * (0.6 + 0.4 * crest) + foam);
        }
        // the sky: night, a little cloud, the halo round the lantern and the beam
        let cl = ((et_n(x / u * 2.6, y / u * 7.) - 0.55) * 3.).clamp(0., 1.);
        let b = beam(x, y);
        let halo = 0.6 * (-((x - lx).powi(2) + (y - ly).powi(2)).sqrt() / (u * 0.06)).exp();
        (1, 0.05 + 0.09 * cl + b * (0.8 + 0.3 * cl) + halo)
    };
    let flat = |_: f32, _: f32| 0_f32;
    let swell = |x: f32, y: f32| ((x / (u * 0.045)) + et_n(x / u * 3., y / u * 9.) * 4.).sin() * u * 0.0028 * (0.3 + (y - hz).max(0.) / (h * 0.25));
    let tower = |x: f32, y: f32| -(1. - ((x - lx) / half(y.max(tt))).powi(2)).max(0.).sqrt() * u * 0.007;
    et_scene(c, k, &scene, &[
        (1, g, 0., Some(-30.), &flat), (2, g * 0.85, 0., None, &swell), (3, g, -38., Some(28.), &flat),
        (4, g * 0.7, 0., Some(90.), &tower), (5, g * 0.6, 0., None, &flat), (6, g * 0.7, 90., None, &flat), (7, g * 0.7, 32., None, &flat),
    ]);
    et_stars(c, k, 140, 52, &|x, y| y < hz - h * 0.08 && beam(x, y) < 0.12 && ((x - lx).powi(2) + (y - ly).powi(2)).sqrt() > u * 0.12);
    // the glazing bars across the lit lantern
    et_light(c, k, lx, ly, u * 0.026, true);
    for dx in [-0.014_f32, 0., 0.014] { c.line(lx + dx * u, h * 0.249, lx + dx * u, ty, Cv::solid(col(k.dark, 0.75)), u * 0.0022); }
    et_frame(c, k, "PHAROS", "Plate II · the lighthouse");
}

/// Plate III — Luna: the full moon over hills, a river carrying its light, a lone tree and three cypresses
fn et_luna(c: &mut Cv, k: &Plate) {
    let (w, h, u) = (c.w, c.h, k.u);
    let g = u * 0.0072;
    let (mx, my, mr) = (w * 0.68, h * 0.245, u * 0.11);
    let ft = |x: f32| h * 0.555 - h * 0.035 * (x / w * 5. + 1.).sin() - h * 0.018 * (x / w * 13.).sin();
    let nt = |x: f32| h * (0.8 - 0.24 * (-((x / w - 0.1) / 0.25).powi(2)).exp()) + (et_n(x / u * 8., 1.) - 0.5) * u * 0.012;
    let river = |y: f32| -> Option<(f32, f32)> {
        let t = (y - h * 0.565) / (h * 0.25);
        if t < 0. { return None; }
        Some((mx * (1. - t) + w * 0.52 * t + (t * 7.).sin() * w * 0.05 * t, u * (0.005 + 0.09 * t.powf(1.5))))
    };
    let (tx, tbase) = (w * 0.17, nt(w * 0.17));
    let crown = [(0., -0.17, 0.055), (-0.045, -0.14, 0.042), (0.048, -0.135, 0.045), (-0.02, -0.21, 0.04), (0.03, -0.2, 0.038)];
    let cyp = [(w * 0.3, 0.12), (w * 0.33, 0.16), (w * 0.355, 0.1)];
    let maria = [(-0.3, -0.25, 0.35, 0.25), (0.22, -0.3, 0.3, 0.22), (0.08, 0.15, 0.42, 0.3), (-0.38, 0.25, 0.2, 0.18)];
    let scene = |x: f32, y: f32| -> (u8, f32) {
        // the lone tree on the near hill, black, its edge silvered on the moon's side
        if (x - tx).abs() < u * 0.006 * (1. + (tbase - y) / (u * 0.4)).min(1.6) && y < tbase + u * 0.01 && y > tbase - u * 0.15 { return (6, 0.04); }
        let depth = crown.iter().map(|&(ox, oy, r)| r * u - ((x - tx - ox * u).powi(2) + (y - tbase - oy * u).powi(2)).sqrt()).fold(f32::MIN, f32::max);
        if depth > 0. {
            let facing = ((x - tx) / (u * 0.07) - (y - tbase + u * 0.17) / (u * 0.12)).clamp(0., 1.);
            let leaf = ((et_n(x / u * 70., y / u * 70.) - 0.5) * 4.).clamp(0., 1.);
            return (6, 0.05 + 0.6 * (-depth / (u * 0.007)).exp() * facing + 0.12 * leaf * facing);
        }
        // three cypresses on the far ridge
        for &(cx, hgt) in &cyp {
            let base = ft(cx) + u * 0.005;
            let t = (y - (base - hgt * h)) / (hgt * h);
            if t > 0. && t < 1. { let hw = u * 0.013 * (PI * t.powf(0.7)).sin().powf(0.6); if (x - cx).abs() < hw { return (7, 0.04 + 0.35 * ((x - cx) / hw).max(0.).powi(3)); } }
        }
        let d = ((x - mx).powi(2) + (y - my).powi(2)).sqrt();
        // a long cloud drawn across the moon's lower edge
        let cy = my + mr * 0.95 + (et_n(x / u * 5., 1.) - 0.5) * u * 0.04;
        let band = (-((y - cy) / (u * 0.016)).powi(2)).exp() * ((x - w * 0.35) / (w * 0.1)).clamp(0., 1.) * ((w * 0.97 - x) / (w * 0.1)).clamp(0., 1.);
        if band > 0.35 && y < h * 0.5 { return (1, 0.12 + 0.55 * (-(d - mr).max(0.) / (u * 0.08)).exp() * (if y < cy { 1. } else { 0.4 })); }
        // the moon: its seas, a bright crater or two, the edge darkening
        if d < mr {
            let (ux, uy) = ((x - mx) / mr, (y - my) / mr);
            if d > mr * 0.965 { return (2, 1.); }
            let mut t = 0.95 - 0.22 * (d / mr).powi(6);
            for &(ax, ay, rx, ry) in &maria { t -= 0.45 * (-(((ux - ax) / rx).powi(2) + ((uy - ay) / ry).powi(2)) * 2.).exp(); }
            for &(ax, ay, rr) in &[(0.25_f32, 0.48_f32, 0.08_f32), (-0.5, -0.05, 0.06), (0.45, -0.1, 0.05)] {
                let cd = ((ux - ax).powi(2) + (uy - ay).powi(2)).sqrt();
                if cd < rr { t = if cd > rr * 0.7 { 1. } else { t - 0.2 }; }
            }
            return (2, t);
        }
        if y < ft(x) { return (1, 0.05 + 0.55 * (-(d - mr) / (u * 0.07)).exp() + 0.25 * (-(d - mr) / (u * 0.3)).exp() + 0.06 * et_n(x / u * 2., y / u * 6.)); }
        if y > nt(x) { return (4, 0.1 + 0.4 * (-(y - nt(x)) / (u * 0.008)).exp() + 0.04 * et_n(x / u * 18., y / u * 18.)); }
        if let Some((rx, rw)) = river(y) {
            if (x - rx).abs() < rw {
                let shimmer = (y / (u * 0.0045)).sin() * 0.5 + 0.5;
                return (5, 0.22 + 0.25 * (1. - ((x - rx) / rw).abs()) + 0.6 * (-((x - mx) / (u * 0.06 + (y - h * 0.565) * 0.2)).powi(2)).exp() * (0.55 + 0.45 * shimmer));
            }
        }
        (3, 0.08 + 0.45 * (-(y - ft(x)) / (u * 0.007)).exp() + 0.14 * et_n(x / u * 10., y / u * 25.) * (1. - (y - ft(x)) / (h * 0.25)).max(0.))
    };
    c.circle(mx, my, mr, Cv::solid(col(k.ink, 0.32)));
    let flat = |_: f32, _: f32| 0_f32;
    let water = |x: f32, y: f32| (x / (u * 0.03) + y / (u * 0.01)).sin() * u * 0.0015;
    let field = |x: f32, _: f32| (x / (u * 0.12)).sin() * u * 0.006;
    let sphere = |x: f32, y: f32| { let (dx, dy) = ((x - mx) / mr, (y - my) / mr); if dx * dx + dy * dy < 1.2 { dy * (1. - dx * dx).max(0.).sqrt() * mr * 0.12 } else { 0. } };
    et_scene(c, k, &scene, &[
        (1, g, 0., Some(-35.), &flat), (2, g * 0.6, 0., None, &sphere), (3, g * 0.85, 6., Some(-28.), &field),
        (4, g, -18., Some(36.), &flat), (5, g * 0.6, 0., None, &water), (6, g * 0.7, 60., None, &flat), (7, g * 0.6, 80., None, &flat),
    ]);
    if let Some(p) = Cv::arc(mx, my, mr, mr, 0., TAU) { c.stroke(&p, Cv::solid(col(k.ink, 0.95)), u * 0.0028); }
    et_stars(c, k, 120, 33, &|x, y| y < ft(x) - u * 0.03 && ((x - mx).powi(2) + (y - my).powi(2)).sqrt() > mr * 2.4);
    let b = c.blend;
    c.blend = BlendMode::Screen;
    c.cloud(mx, my, mr * 2.6, k.glow, 0.14);
    c.blend = b;
    et_frame(c, k, "LVNA", "Plate III · the moon");
}

/// Plate IV — Memento mori: a skull on a closed book, an hourglass, a candle burned to a stub, a letter left
fn et_vanitas(c: &mut Cv, k: &Plate) {
    let (w, h, u) = (c.w, c.h, k.u);
    let g = u * 0.0072;
    let (lx, ly) = (w * 0.8, h * 0.505);
    let lamp = |x: f32, y: f32| et_lamp(x, y, lx, ly, u * 0.42);
    let (tb, tf) = (h * 0.62, h * 0.685);
    let (scx, top_y, cw, hy) = (lx, h * 0.548, u * 0.028, h * 0.655);
    let (bx0, bx1, bft, bb, dpt, off) = (w * 0.27, w * 0.6, h * 0.575, h * 0.66, u * 0.035, u * 0.03);
    let (sx, sy) = (w * 0.43, bft - dpt * 0.5 - u * 0.2);
    let ld = { let (dx, dy) = (lx - sx, ly - sy); let m = (dx * dx + dy * dy).sqrt(); (dx / m, dy / m * 0.6 - 0.35, 0.75) };
    let (gx, gt, gb) = (w * 0.155, h * 0.415, h * 0.66);
    let gw = |y: f32| { let t = (y - gt - u * 0.015) / (gb - gt - u * 0.03); u * 0.052 * (PI * t).sin().abs().powf(0.55).max(0.1) };
    let skull = |x: f32, y: f32| -> Option<f32> {
        let lit = |x: f32, y: f32| et_ball(x, y, sx, sy + u * 0.03, u * 0.14, u * 0.19, ld).unwrap_or(0.1);
        let lf = (lamp(x, y) * 2.6).min(1.);
        let shade = |t: f32| 0.07 + 0.88 * t * lf;
        // eye sockets, deep, a lit rim on the inner edge
        for side in [-1_f32, 1.] {
            let (ex, ey) = (sx + side * u * 0.048, sy + u * 0.035);
            let e = ((x - ex) / (u * 0.037)).powi(2) + ((y - ey) / (u * 0.031)).powi(2);
            if e < 1. { return Some(if e > 0.8 && (x - ex) * -side < 0. { shade(0.6) } else { 0.02 }); }
        }
        // the nose, an inverted heart of dark
        let ny = (y - (sy + u * 0.07)) / (u * 0.045);
        if ny > 0. && ny < 1. && (x - sx).abs() < u * (0.004 + 0.015 * ny) { return Some(0.02); }
        // the teeth, upper and lower, and the line where they meet
        if y > sy + u * 0.128 && y < sy + u * 0.168 && (x - sx).abs() < u * 0.055 {
            if (y - (sy + u * 0.148)).abs() < u * 0.0025 { return Some(0.02); }
            let f = ((x - sx) / (u * 0.0135)).fract().abs();
            return Some(if f < 0.14 { 0.04 } else { shade(lit(x, y) * 0.9 + 0.1) });
        }
        // the cheekbones
        for side in [-1_f32, 1.] {
            if ((x - (sx + side * u * 0.088)) / (u * 0.036)).powi(2) + ((y - (sy + u * 0.06)) / (u * 0.024)).powi(2) < 1. { return Some(shade((lit(x, y) + 0.15).min(1.))); }
        }
        // the cranium, the face under it, the jaw
        let cran = ((x - sx) / (u * 0.128)).powi(2) + ((y - (sy - u * 0.02)) / (u * 0.118)).powi(2) < 1.;
        let face = y > sy && y < sy + u * 0.13 && (x - sx).abs() < u * 0.09 * (1. - (y - sy) / (u * 0.5));
        let jaw = y >= sy + u * 0.13 && y < sy + u * 0.2 && (x - sx).abs() < u * 0.072 - (y - sy - u * 0.13) * 0.45;
        if cran || face || jaw {
            let temple = ((x - sx).abs() > u * 0.095 && y > sy - u * 0.01 && y < sy + u * 0.05) as i32 as f32;
            return Some(shade((lit(x, y) - 0.25 * temple).max(0.)));
        }
        None
    };
    let scene = |x: f32, y: f32| -> (u8, f32) {
        let l = lamp(x, y);
        if let Some(t) = skull(x, y) { return (5, t); }
        // the candle stub on its dish
        let ex = ((x - scx) / cw).powi(2) + ((y - top_y) / (cw * 0.3)).powi(2);
        if ex < 1. { return (7, 0.95); }
        if (x - scx).abs() < cw && y > top_y && y < hy { let nx = (x - scx) / cw; return (7, 0.25 + 0.65 * (1. - nx * nx).sqrt()); }
        let e = ((x - scx) / (u * 0.08)).powi(2) + ((y - hy) / (u * 0.017)).powi(2);
        if e < 1. { return (8, if e > 0.6 { 0.9 } else { 0.55 }); }
        // the hourglass: two plates, three posts, the glass, the sand mostly run through
        if (x - gx).abs() < u * 0.064 && ((y > gt && y < gt + u * 0.015) || (y > gb - u * 0.015 && y < gb)) { return (9, 0.2 + 0.6 * ((x - gx) / (u * 0.064) * 0.5 + 0.5) * (l * 2.).min(1.)); }
        for px in [-0.054_f32, 0.054, 0.012] {
            if (x - (gx + px * u)).abs() < u * 0.0045 && y > gt && y < gb { return (9, 0.15 + 0.6 * ((x - (gx + px * u)) / (u * 0.0045) * 0.5 + 0.5) * (l * 2.).min(1.)); }
        }
        if y > gt && y < gb && (x - gx).abs() < gw(y) {
            let f = (x - gx) / gw(y);
            let neck = (gt + gb) / 2.;
            let sand = (y > h * 0.6 && y < gb) || (y > neck - u * 0.04 && y < neck) || ((x - gx).abs() < u * 0.0015 && y > neck);
            if sand { return (10, 0.3 + 0.45 * (f * 0.5 + 0.5) * (l * 2.).min(1.)); }
            return (10, 0.04 + if f > 0.45 && f < 0.7 { 0.8 * (l * 2.).min(1.) } else { 0. });
        }
        // the book under the skull
        if y > bft - dpt && y <= bft { let s = (bft - y) / dpt * off; if x > bx0 + s && x < bx1 + s { return (11, 0.75 * (l * 1.6).min(1.)); } }
        if x >= bx1 && x < bx1 + off { let s = (x - bx1) / off * dpt; if y > bft - s && y < bb - s { return (12, 0.9 * (l * 1.6).min(1.)); } }
        if x > bx0 && x < bx1 && y > bft && y < bb {
            let f = (y - bft) / (bb - bft);
            return (13, if f < 0.15 || f > 0.85 { 0.28 * (l * 1.6).min(1.) } else { 0.62 * (l * 1.6).min(1.) });
        }
        // a letter left on the table, hanging over its edge
        let (px0, px1) = (w * 0.6, w * 0.72);
        if x > px0 && x < px1 && y > tb + u * 0.03 && y < tf + u * 0.06 + (x - px0) * 0.05 {
            if y > tf { return (14, 0.35 * (l * 1.6).min(1.)); }
            return (14, 0.85 * (l * 1.4).min(1.) * if ((y - tb) / (u * 0.008)).fract() < 0.18 && x > px0 + u * 0.015 && x < px1 - u * 0.02 { 0.5 } else { 1. });
        }
        // the wall with the skull's shadow on it, the table, its edge and front
        if y < tb {
            let sd = ((x - (sx - u * 0.2)) / (u * 0.17)).powi(2) + ((y - (sy + u * 0.05)) / (u * 0.19)).powi(2);
            let sh = if sd < 1. { 0.45 + 0.55 * sd.powi(3) } else { 1. };
            return (1, (l * 1.05 * sh * (0.9 + 0.1 * et_n(x / u * 9., y / u * 9.))).min(1.));
        }
        if y < tf {
            let mut sh = 1.;
            if x < bx0 && x > bx0 - u * 0.2 && y < bb { sh = 0.3 + 0.7 * ((bx0 - x) / (u * 0.2)).powi(2); }
            if x < gx - u * 0.02 && x > gx - u * 0.17 && y < gb + u * 0.006 { sh = sh.min(0.35); }
            return (2, (l * 1.15 * sh).min(1.));
        }
        if y < tf + u * 0.006 { return (2, 0.5 * l + 0.3); }
        (3, l * 0.38)
    };
    let flat = |_: f32, _: f32| 0_f32;
    let round = |x: f32, _: f32| -(1. - ((x - scx) / cw).powi(2)).max(0.).sqrt() * u * 0.006;
    let glass = |x: f32, y: f32| -(1. - ((x - gx) / gw(y.clamp(gt, gb))).powi(2)).max(0.).sqrt() * u * 0.006;
    let bone = |x: f32, y: f32| ((x - sx) / (u * 0.13)).clamp(-1., 1.) * ((y - sy) / (u * 0.2)) * u * 0.01;
    et_scene(c, k, &scene, &[
        (1, g, 0., Some(65.), &flat), (2, g * 0.85, -3., Some(40.), &flat), (3, g, 90., None, &flat),
        (5, g * 0.62, -28., Some(52.), &bone), (7, g * 0.75, 0., None, &round), (8, g * 0.7, 0., None, &flat),
        (9, g * 0.6, 90., None, &flat), (10, g * 0.6, 0., Some(90.), &glass), (11, g * 0.8, -12., Some(50.), &flat),
        (12, g * 0.8, 90., None, &flat), (13, g * 0.55, 0., None, &flat), (14, g * 0.6, 2., None, &flat),
    ]);
    et_flame(c, k, lx, ly + u * 0.03, u * 0.05);
    et_light(c, k, lx, ly + u * 0.01, u * 0.015, true);
    et_frame(c, k, "MEMENTO MORI", "Plate IV · still life");
}

/// Plate V — Aurora: the sun coming up in the notch between two mountains, a river in the valley, pines in front
fn et_aurora(c: &mut Cv, k: &Plate) {
    let (w, h, u) = (c.w, c.h, k.u);
    let g = u * 0.0072;
    let (sx, sy, sr) = (w * 0.5, h * 0.5, u * 0.05);
    let mtop = |x: f32| -> f32 {
        let lin = |a: (f32, f32), b: (f32, f32), x: f32| a.1 + (b.1 - a.1) * ((x - a.0) / (b.0 - a.0));
        let base = if x < w * 0.27 { lin((0., h * 0.36), (w * 0.27, h * 0.2), x) }
            else if x < w * 0.5 { lin((w * 0.27, h * 0.2), (w * 0.5, h * 0.5), x) }
            else if x < w * 0.74 { lin((w * 0.5, h * 0.5), (w * 0.74, h * 0.16), x) }
            else { lin((w * 0.74, h * 0.16), (w, h * 0.33), x) };
        let notch = ((x - sx).abs() / (u * 0.06)).min(1.);
        base + ((et_n(x / u * 10., 2.) - 0.5) * u * 0.08 + (et_n(x / u * 31., 5.) - 0.5) * u * 0.022) * notch
    };
    let hill = |x: f32| h * 0.575 + h * 0.025 * (x / w * 7.).sin() + (et_n(x / u * 12., 4.) - 0.5) * u * 0.012;
    let river = |y: f32| -> Option<(f32, f32)> {
        let t = (y - h * 0.565) / (h * 0.23);
        if t < 0. { return None; }
        Some((sx + (t * 5.).sin() * w * 0.09 * t - w * 0.08 * t, u * (0.003 + 0.065 * t.powf(1.6))))
    };
    let mut r = Rng::new(905);
    let pines: Vec<(f32, f32, f32)> = (0..19).map(|i| {
        let x = w * (0.02 + i as f32 * 0.054 + (r.f() - 0.5) * 0.02);
        let edge = (x / w - 0.5).abs() * 2.;
        (x, h * (0.81 + r.f() * 0.02), h * (0.07 + 0.2 * edge.powf(1.3) + r.f() * 0.05))
    }).collect();
    let scene = |x: f32, y: f32| -> (u8, f32) {
        // the pines along the front, black, each edge facing the sun just caught
        for &(px, base, hgt) in &pines {
            let t = (base - y) / hgt;
            if (0. ..1.).contains(&t) {
                let hw = u * 0.04 * (1. - t) * (0.6 + 0.4 * (t * 6.).fract()) + if t < 0.12 { 0. } else { 0. };
                let trunk = (x - px).abs() < u * 0.004;
                if (x - px).abs() < hw || trunk {
                    let facing = (x - px) * (sx - px) > 0.;
                    return (6, 0.03 + if facing { 0.45 * (-(hw - (x - px).abs()) / (u * 0.004)).exp() } else { 0. });
                }
            }
        }
        let d = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
        let glow = (-(x - sx).abs() / (u * 0.45)).exp();
        let mt = mtop(x);
        if y < mt {
            if d < sr { return (0, 0.); }
            let a = (y - sy).atan2(x - sx);
            let rays = (a * 18.).cos().max(0.).powi(8) * (-d / (u * 0.7)).exp() * 0.3;
            let cloud = ((et_n(x / u * 1.8, y / u * 9.) - 0.6) * 4.).clamp(0., 1.);
            return (1, 0.1 + 0.85 * (-(d - sr) / (u * 0.2)).exp() + rays + cloud * (0.15 + 0.5 * glow));
        }
        if y < hill(x) {
            let rim = (-(y - mt) / (u * 0.009)).exp();
            let inner = (x > w * 0.27 && x < sx) || (x > sx && x < w * 0.74);
            let left = x < sx;
            return (if left { 2 } else { 3 }, 0.06 + 0.6 * rim * glow + if inner { 0.12 * glow } else { 0. } + 0.14 * et_n(x / u * 20. + y / u * 6., y / u * 20.));
        }
        if let Some((rx, rw)) = river(y) {
            if (x - rx).abs() < rw {
                let shimmer = (y / (u * 0.004)).sin() * 0.5 + 0.5;
                return (5, 0.3 + 0.65 * (1. - ((x - rx) / rw).abs()) * (0.6 + 0.4 * shimmer));
            }
        }
        (4, 0.05 + 0.32 * (-(y - hill(x)) / (u * 0.008)).exp() * glow + 0.1 * et_n(x / u * 16., y / u * 16.))
    };
    let flat = |_: f32, _: f32| 0_f32;
    let water = |x: f32, y: f32| (x / (u * 0.03) + y / (u * 0.008)).sin() * u * 0.0014;
    et_scene(c, k, &scene, &[
        (1, g, 0., Some(-40.), &flat), (2, g * 0.85, -52., Some(30.), &flat), (3, g * 0.85, 52., Some(-30.), &flat),
        (4, g * 0.9, 4., Some(-35.), &flat), (5, g * 0.6, 0., None, &water), (6, g * 0.7, 70., None, &flat),
    ]);
    // birds going over, a stroke each wing
    for (bx, by, s) in [(0.62, 0.28, 1.), (0.66, 0.25, 0.8), (0.69, 0.3, 0.9), (0.715, 0.265, 0.7), (0.6, 0.33, 0.6)] {
        let (x, y, s) = (w * bx, h * by, u * 0.012 * s);
        c.line(x - s, y - s * 0.35, x, y + s * 0.15, Cv::solid(col(k.ink, 0.85)), u * 0.0018);
        c.line(x, y + s * 0.15, x + s, y - s * 0.4, Cv::solid(col(k.ink, 0.85)), u * 0.0018);
    }
    // the sun's disc, cut as rings round a bright core, half behind the mountains
    let mut pb = PathBuilder::new();
    pb.push_circle(sx, sy, sr);
    if let Some(p) = pb.finish() {
        let mut sky: Vec<(f32, f32)> = (0..=40).map(|i| { let x = sx - sr * 1.5 + sr * 3. * i as f32 / 40.; (x, mtop(x)) }).collect();
        sky.push((sx + sr * 1.5, sy - sr * 2.));
        sky.push((sx - sr * 1.5, sy - sr * 2.));
        if let Some(cl) = mq_poly(&sky) { c.clip_path(&cl); }
        c.fill(&p, Cv::solid(col(k.ink, 0.95)));
        c.clip = None;
    }
    et_light(c, k, sx, sy - sr * 0.2, u * 0.03, true);
    et_frame(c, k, "AVRORA", "Plate V · the valley at dawn");
}

/// the press: the plate's last wiping (a faint tone left in the black), its grain, the edge darkening
fn et_finish(c: &mut Cv, k: &Plate, seed: u32) {
    let (w, h, u) = (c.w, c.h, k.u);
    let mut r = Rng::new(seed as u64 + 31);
    let b = c.blend;
    c.blend = BlendMode::Screen;
    for _ in 0..6 { let (x, y) = (r.f() * w, r.f() * h); c.cloud(x, y, (0.2 + r.f() * 0.3) * w, k.ink, 0.018); }
    c.blend = b;
    for _ in 0..(w * h / 9000.) as usize { let (x, y) = (r.f() * w, r.f() * h); c.circle(x, y, u * (0.0004 + r.f() * 0.0012), Cv::solid(col(k.ink, 0.1 + r.f() * 0.25))); }
    c.vignette(BLACK, 0.4);
    c.noise(0.035, k.dark, seed);
}

fn etch(c: &mut Cv, t: &Theme, i: usize) {
    let k = et_plate(c, t);
    c.fill_all(Cv::solid(col(k.dark, 1.)));
    match i {
        0 => et_candle(c, &k),
        1 => et_pharos(c, &k),
        2 => et_luna(c, &k),
        3 => et_vanitas(c, &k),
        _ => et_aurora(c, &k),
    }
    c.reset();
    et_finish(c, &k, 170 + i as u32);
}

/* ── Dither: a 1-bit screen, five pictures in black and white dots, Bayer-ordered ─────────────────────────── */
// Every picture is drawn small (one dot per four screen pixels at 1080p) as a smooth grey tone, then dithered with an
// 8×8 Bayer matrix to pure ink or pure ground, and blown up with hard edges. The window chrome on it (an info box with
// a striped title bar, the plate's 8×8 icon, a checker progress bar) is drawn on a second layer that is thresholded
// hard, so lines and type stay crisp while light falls off in dots.

/// the screen: its two colours, its dot size, and the small canvases the picture and the chrome are drawn on
struct Screen { ink: Rgb, ground: Rgb, s: u32, lw: u32, lh: u32, wn: f32, mono: String, body: String }
fn dh_screen(c: &Cv, t: &Theme) -> Screen {
    let p = t.palette;
    let (ink, ground) = if p.ink.luma() >= p.bg.luma() { (p.ink, p.bg) } else { (p.bg, p.ink) };
    let (w, h) = (c.px.width(), c.px.height());
    let s = ((h as f32).min(w as f32 * 0.5625) / 270.).round().max(1.) as u32;
    let (lw, lh) = (w.div_ceil(s), h.div_ceil(s));
    Screen { ink, ground, s, lw, lh, wn: lw as f32 / (lh as f32).min(lw as f32 * 0.5625), mono: t.display.clone(), body: t.display.clone() }
}
const BAYER: [[u8; 8]; 8] = [[0, 32, 8, 40, 2, 34, 10, 42], [48, 16, 56, 24, 50, 18, 58, 26], [12, 44, 4, 36, 14, 46, 6, 38], [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41], [51, 19, 59, 27, 49, 17, 57, 25], [15, 47, 7, 39, 13, 45, 5, 37], [63, 31, 55, 23, 61, 29, 53, 21]];
/// the five icons, 8×8, as the bar draws them too: sun, moon, star, eye, door
const DH_ICONS: [[&str; 8]; 5] = [
    ["...#....", ".#.#.#..", "..###...", "#######.", "..###...", ".#.#.#..", "...#....", "........"],
    ["..###...", ".##.....", "##......", "##......", "##......", ".##.....", "..###...", "........"],
    ["...#....", "...#....", "..###...", "#######.", ".#####..", ".##.##..", "#.....#.", "........"],
    ["........", "..####..", ".#....#.", "#..##..#", "#..##..#", ".#....#.", "..####..", "........"],
    [".######.", ".#....#.", ".#....#.", ".#....#.", ".#..#.#.", ".#....#.", ".#....#.", "########"],
];

/// value noise and its fractal sum, for clouds and dust (0..1)
fn dh_hash(i: i32, j: i32, s: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(374761393) ^ (j as u32).wrapping_mul(668265263) ^ s.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}
fn dh_noise(x: f32, y: f32, s: u32) -> f32 {
    let (i, j) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3. - 2. * fx), fy * fy * (3. - 2. * fy));
    let (a, b, c, d) = (dh_hash(i, j, s), dh_hash(i + 1, j, s), dh_hash(i, j + 1, s), dh_hash(i + 1, j + 1, s));
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}
fn dh_fbm(x: f32, y: f32, s: u32) -> f32 {
    let (mut v, mut a, mut x, mut y) = (0., 0.5, x, y);
    for o in 0..5 { v += a * dh_noise(x, y, s + o); x *= 2.03; y *= 2.03; a *= 0.5; }
    v / 0.96875
}
fn dh_step(e0: f32, e1: f32, x: f32) -> f32 { let t = ((x - e0) / (e1 - e0)).clamp(0., 1.); t * t * (3. - 2. * t) }

/// one dot of the chrome layer, hard: on (ink) or off (ground)
fn dh_px(ui: &mut Cv, k: &Screen, x: f32, y: f32, w: f32, h: f32, on: bool) {
    ui.fill_rect(x.round(), y.round(), w.round().max(1.), h.round().max(1.), Cv::solid(col(if on { WHITE } else { BLACK }, 1.)));
    let _ = k;
}
/// an icon from its 8×8 grid, `z` dots to a cell
fn dh_icon(ui: &mut Cv, k: &Screen, n: usize, x: f32, y: f32, z: f32, on: bool) {
    for (j, row) in DH_ICONS[n % 5].iter().enumerate() {
        for (i, ch) in row.chars().enumerate() { if ch == '#' { dh_px(ui, k, x + i as f32 * z, y + j as f32 * z, z, z, on); } }
    }
}
/// a window of the old kind: a box with a 1-dot border and a 2-dot drop shadow, the title bar striped across with the
/// title knocked out in its middle and a close box at its left; returns the content's top
fn dh_window(ui: &mut Cv, k: &Screen, x: f32, y: f32, w: f32, h: f32, title: &str) -> f32 {
    dh_px(ui, k, x + 2., y + 2., w, h, true);
    dh_px(ui, k, x, y, w, h, true);
    dh_px(ui, k, x + 1., y + 1., w - 2., h - 2., false);
    // the title bar: six stripes, the close box, the title in a gap
    for r in 0..6 { dh_px(ui, k, x + 2., y + 3. + r as f32 * 2., w - 4., 1., true); }
    dh_px(ui, k, x + 7., y + 2., 11., 13., false);
    dh_px(ui, k, x + 8., y + 4., 9., 9., true);
    dh_px(ui, k, x + 9., y + 5., 7., 7., false);
    let tw = crate::paint::measure(title, &k.mono, 8.).ceil() + 8.;
    dh_px(ui, k, x + (w - tw) / 2., y + 2., tw, 13., false);
    ui.text(title, &k.mono, 8., (x + (w - tw) / 2. + 4.).round(), y + 12., col(WHITE, 1.));
    dh_px(ui, k, x + 1., y + 16., w - 2., 1., true);
    y + 17.
}
/// the info box every picture carries: the icon at three dots a cell, the file name, its size, and how far along the
/// set it is as a checkerboard bar
fn dh_info(ui: &mut Cv, k: &Screen, i: usize, name: &str, line: &str) {
    let (w, h) = (176., 66.);
    let (x, y) = (k.lw as f32 - w - 14., 22.);
    let top = dh_window(ui, k, x, y, w, h, "GET INFO");
    dh_px(ui, k, x + 1., top, w - 2., h - 18., false);
    dh_icon(ui, k, i, x + 9., top + 7., 3., true);
    ui.text(name, &k.mono, 8., x + 42., top + 13., col(WHITE, 1.));
    ui.text(line, &k.body, 8., x + 42., top + 25., col(WHITE, 1.));
    // the bar: a 1-dot frame, filled solid up to this picture, checkered for the rest
    let (bx, by, bw) = (x + 42., top + 32., w - 54.);
    dh_px(ui, k, bx, by, bw, 9., true);
    dh_px(ui, k, bx + 1., by + 1., bw - 2., 7., false);
    let fill = ((bw - 4.) * (i + 1) as f32 / 5.).round();
    dh_px(ui, k, bx + 2., by + 2., fill, 5., true);
    for yy in 0..5 { for xx in (fill as i32)..((bw - 4.) as i32) { if (xx + yy) % 2 == 0 { dh_px(ui, k, bx + 2. + xx as f32, by + 2. + yy as f32, 1., 1., true); } } }
}
/// the press of the screen: the grey picture dithered dot by dot through the Bayer matrix, the chrome laid over it
/// hard, each dot blown up to `s`×`s` screen pixels in the theme's two colours
fn dh_print(c: &mut Cv, k: &Screen, tone: &dyn Fn(f32, f32) -> f32, ui: &Cv) {
    let (lw, lh, s) = (k.lw as usize, k.lh as usize, k.s as usize);
    let (w, h) = (c.px.width() as usize, c.px.height() as usize);
    let un = (k.lh as f32).min(k.lw as f32 * 0.5625);
    let on = tiny_skia::PremultipliedColorU8::from_rgba(k.ink.0, k.ink.1, k.ink.2, 255).unwrap();
    let off = tiny_skia::PremultipliedColorU8::from_rgba(k.ground.0, k.ground.1, k.ground.2, 255).unwrap();
    let uip = ui.px.pixels();
    let mut dots = vec![false; lw * lh];
    for y in 0..lh {
        for x in 0..lw {
            let p = uip[y * lw + x];
            dots[y * lw + x] = if p.alpha() >= 128 { p.red() as u32 * 2 >= p.alpha() as u32 } else {
                tone((x as f32 + 0.5) / un, (y as f32 + 0.5) / un) * 64. > BAYER[y % 8][x % 8] as f32 + 0.5
            };
        }
    }
    let px = c.px.pixels_mut();
    for y in 0..h { let ly = (y / s).min(lh - 1); for x in 0..w { px[y * w + x] = if dots[ly * lw + (x / s).min(lw - 1)] { on } else { off }; } }
}

/// 1 — SUN: the sun behind a bank of cloud with its lining lit, its beams fanning down over the sea, a road of glints
/// on the water, an island black on the horizon and gulls in the light
fn dh_beams(c: &mut Cv, k: &Screen) {
    let wn = k.wn;
    let (sx, sy, hz) = (wn * 0.4, 0.21, 0.66);
    let cloud = |x: f32, y: f32| dh_fbm(x * 2.4, y * 6.5, 11) * 0.9 + 0.62 * (-((y - 0.21) / 0.1).powi(2)).exp() - 0.2;
    let island = |x: f32| hz - 0.04 * (1. - ((x - wn * 0.13) / 0.2).powi(2)).max(0.).sqrt() - 0.018 * (1. - ((x - wn * 0.2) / 0.05).powi(2)).max(0.);
    let tone = |x: f32, y: f32| -> f32 {
        let d = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
        let glow = (-d / 0.22).exp();
        // the beams: wedges out of the sun, only below the bank, fading as they go
        let a = (y - sy).atan2(x - sx);
        let wedge = dh_step(0.25, 0.45, (a * 26. + 2.5 * dh_noise(a * 7., 0.3, 3)).sin()) * dh_step(0.3, 0.38, y) * (-d / 0.7).exp();
        if y > hz {
            let w = 0.025 + 0.45 * (y - hz);
            let road = (-((x - sx) / w).powi(2)).exp();
            let glint = dh_step(0.58, 0.68, dh_noise(x * 45., y * 260., 5));
            return 0.05 + 0.08 * (y - hz) / (1. - hz) + road * (0.12 + 0.85 * glint) + 0.12 * wedge;
        }
        if y > island(x) { return 0.; }
        let sky = 0.04 + 0.32 * ((y - 0.02) / (hz - 0.02)).clamp(0., 1.).powf(1.6) + 0.55 * glow + if d < 0.035 { 1. } else { 0. };
        let n = cloud(x, y);
        let cover = dh_step(0.5, 0.53, n);
        // the lining: a band just inside the cloud's edge, bright as the sun is near
        let rim = cover * (1. - dh_step(0.53, 0.6, n));
        sky * (1. - cover) + (0.06 + 0.1 * glow) * cover + rim * (0.15 + 1.4 * glow) + 0.35 * wedge * (1. - cover)
    };
    let mut ui = Cv::new(k.lw, k.lh);
    let (fw, fh) = (k.lw as f32, k.lh as f32);
    // gulls in the beams, black, a crooked V each
    for (bx, by) in [(0.56, 0.44), (0.6, 0.41), (0.635, 0.465)] {
        let (x, y) = (bx * fw, by * fh);
        for (dx, dy, w) in [(-4., -2., 2.), (-2., -1., 2.), (0., 0., 1.), (1., -1., 2.), (3., -2., 2.)] { dh_px(&mut ui, k, x + dx, y + dy, w, 1., false); }
    }
    dh_info(&mut ui, k, 0, "BEAMS.PIC", "SUN / 1 BIT / 8X8");
    dh_print(c, k, &tone, &ui);
}

/// 2 — MOON: the full moon in a sky of heaped cloud, their edges lit on its side; a house on the far ridge, one
/// window lit
fn dh_clouds(c: &mut Cv, k: &Screen) {
    let wn = k.wn;
    let (mx, my, mr) = (wn * 0.36, 0.3, 0.075);
    let hill = |x: f32| 0.84 + 0.035 * (x / wn * 5. + 1.).sin() - 0.06 * (-((x - wn * 0.7) / 0.2).powi(2)).exp();
    let cloud = |x: f32, y: f32| dh_fbm(x * 1.9 + 3., y * 4.8, 21) + 0.3 * (-((y - 0.58) / 0.12).powi(2)).exp() + 0.22 * (-((y - 0.12) / 0.07).powi(2)).exp() - 0.08;
    let tone = |x: f32, y: f32| -> f32 {
        let hy = hill(x);
        if y > hy { return 0.; }
        let d = ((x - mx).powi(2) + (y - my).powi(2)).sqrt();
        let halo = 0.3 * (-(d - mr).max(0.) / 0.04).exp() + 0.16 * (-(d - mr).max(0.) / 0.3).exp();
        let n = cloud(x, y);
        let cover = dh_step(0.5, 0.56, n) * dh_step(mr * 1.1, mr * 1.9, d);
        // the side toward the moon is where the cloud thins: lit there, dark in its body
        let (vx, vy) = ((mx - x) / d.max(1e-3), (my - y) / d.max(1e-3));
        let lit = ((n - cloud(x + vx * 0.02, y + vy * 0.02)) * 12.).clamp(0., 1.);
        let cl = 0.1 + 0.16 * (-d / 0.5).exp() + 0.08 * dh_noise(x * 14., y * 30., 23) + lit * (0.35 + 1.5 * halo).min(0.9);
        let sky = if d < mr {
            let cr = dh_fbm((x - mx) * 30., (y - my) * 30., 7);
            0.98 - 0.25 * dh_step(0.55, 0.7, cr) - 0.15 * (d / mr).powi(4)
        } else { 0.02 + halo + 0.2 * (-(hy - y) / 0.1).exp() };
        sky * (1. - cover) + cl * cover
    };
    let mut ui = Cv::new(k.lw, k.lh);
    let (fw, fh) = (k.lw as f32, k.lh as f32);
    let un = fh.min(fw * 0.5625);
    let mut r = Rng::new(71);
    for _ in 0..90 {
        let (x, y) = (r.f() * fw, r.f() * fh * 0.75);
        let (nx, ny) = (x / un, y / un);
        if cloud(nx, ny) > 0.46 || ((nx - mx).powi(2) + (ny - my).powi(2)).sqrt() < mr * 2.2 { continue; }
        dh_px(&mut ui, k, x, y, 1., 1., true);
    }
    // the house on the ridge, black against the haze: walls, a pitched roof, a chimney, one window lit
    let hx = wn * 0.7 * un;
    let hy = hill(wn * 0.7) * un + 1.;
    dh_px(&mut ui, k, hx - 10., hy - 12., 20., 12., false);
    for j in 0..9 { dh_px(&mut ui, k, hx - 12. + j as f32 * 1.4, hy - 13. - j as f32, 24. - j as f32 * 2.8, 1., false); }
    dh_px(&mut ui, k, hx + 4., hy - 22., 3., 6., false);
    dh_px(&mut ui, k, hx - 6., hy - 9., 4., 4., true);
    dh_px(&mut ui, k, hx - 4.5, hy - 9., 1., 4., false);
    dh_info(&mut ui, k, 1, "CLOUDS.PIC", "MOON / 1 BIT / 8X8");
    dh_print(c, k, &tone, &ui);
}

/// 3 — STAR: a spiral of stars turning round its bright core, dust lanes down the arms, single stars all round
fn dh_spiral(c: &mut Cv, k: &Screen) {
    let wn = k.wn;
    let (cx, cy) = (wn * 0.5, 0.52);
    let tone = |x: f32, y: f32| -> f32 {
        // the disc seen a little from the side
        let (dx, dy) = ((x - cx) / 1.5, (y - cy) / 0.62 / 1.5);
        let (dx, dy) = (dx * 0.94 - dy * 0.34, dx * 0.34 + dy * 0.94);
        let r = (dx * dx + dy * dy).sqrt();
        let th = dy.atan2(dx);
        let ph = th - (r / 0.015).max(1.).ln() * 2.1;
        let arm = ((2. * ph).cos() * 0.5 + 0.5).powf(2.5);
        let lane = ((2. * ph + 0.7).cos() * 0.5 + 0.5).powf(8.);
        let dust = dh_fbm(x * 18., y * 18., 31);
        0.02 + arm * (-r / 0.3).exp() * (0.55 + 0.6 * dust) - 0.25 * lane * (-r / 0.25).exp() + 0.95 * (-r / 0.045).exp() + 0.05 * (-r / 0.6).exp()
    };
    let mut ui = Cv::new(k.lw, k.lh);
    let (fw, fh) = (k.lw as f32, k.lh as f32);
    let mut r = Rng::new(303);
    for n in 0..160 {
        let (x, y) = (r.f() * fw, r.f() * fh);
        dh_px(&mut ui, k, x, y, 1., 1., true);
        // a few bright ones, a cross of five dots
        if n % 23 == 0 { for (dx, dy) in [(-1., 0.), (1., 0.), (0., -1.), (0., 1.)] { dh_px(&mut ui, k, x + dx, y + dy, 1., 1., true); } }
    }
    dh_info(&mut ui, k, 2, "SPIRAL.PIC", "STAR / 1 BIT / 8X8");
    dh_print(c, k, &tone, &ui);
}

/// 4 — EYE: a rose window in a dark nave, its light coming down in shafts across a chequered floor
fn dh_window_art(c: &mut Cv, k: &Screen) {
    let wn = k.wn;
    let (cx, cy, rr) = (wn * 0.5, 0.33, 0.2);
    let dir = (0.32_f32, 1_f32);
    let rose = |x: f32, y: f32| -> Option<f32> {
        let (dx, dy) = (x - cx, y - cy);
        let d = (dx * dx + dy * dy).sqrt() / rr;
        if d > 1. { return None; }
        let th = dy.atan2(dx);
        let spoke = ((th / TAU * 12.).rem_euclid(1.) - 0.5).abs() * TAU / 12. * d * rr;
        if d > 0.9 || (d > 0.3 && d < 0.36) || (d < 0.12) { return Some(if d < 0.12 { 0.45 } else { 0.18 }); }
        if spoke < 0.005 && d > 0.36 { return Some(0.12); }
        // the petals: a circle in each light of the outer ring
        let pa = ((th / TAU * 12.).floor() + 0.5) / 12. * TAU;
        let (px, py) = (cx + pa.cos() * rr * 0.63, cy + pa.sin() * rr * 0.63);
        let pd = ((x - px).powi(2) + (y - py).powi(2)).sqrt() / (rr * 0.17);
        if (pd - 1.).abs() < 0.12 { return Some(0.15); }
        Some(0.6 + 0.35 * dh_noise(th * 6. + d * 9., d * 14., 41))
    };
    let tone = |x: f32, y: f32| -> f32 {
        if let Some(t) = rose(x, y) { return t; }
        // two lancets each side, pointed, a mullion down each
        for side in [-1_f32, 1.] {
            let lx = cx + side * wn * 0.27;
            let (hw, top, bot) = (0.045, 0.3, 0.62);
            let dx = (x - lx).abs();
            let arch = top + 0.07 * (1. - (1. - dx / hw).max(0.).powi(2)).sqrt();
            if dx < hw && y > top + 0.07 - (0.07 * (1. - (dx / hw).powi(2)).max(0.).sqrt()) && y < bot {
                let _ = arch;
                return if dx < 0.004 { 0.12 } else { 0.55 + 0.3 * dh_noise(x * 40., y * 20., 43) };
            }
        }
        // the columns, black, a lit edge on the side toward the rose
        for side in [-1_f32, 1.] {
            let px = cx + side * wn * 0.42;
            if (x - px).abs() < 0.07 && y < 0.95 { return if (x - px) * -side > 0.055 { 0.4 } else { 0.02 }; }
        }
        let floor = 0.8;
        // the shafts: follow each point back up the light to the window's plane and see if it lands in the rose
        let t = (y - cy) / dir.1;
        let (qx, qy) = (x - dir.0 * t, y - dir.1 * t);
        let shaft = if t > 0. && ((qx - cx).powi(2) + (qy - cy).powi(2)).sqrt() < rr * 0.88 { 1. } else { 0. };
        if y > floor {
            let (u, v) = ((x - cx) / (y - floor + 0.06), 1. / (y - floor + 0.06));
            let tile = (((u * 3.).floor() + (v * 0.6).floor()) as i32).rem_euclid(2) as f32;
            // where the shafts reach the floor: the window laid down on the tiles
            let (fx, fy) = (x - dir.0 * ((floor - cy) / dir.1) , floor);
            let _ = (fx, fy);
            return 0.03 + 0.13 * tile + shaft * (0.4 + 0.3 * tile);
        }
        let dust = dh_fbm(x * 9., y * 9., 47);
        0.03 + 0.04 * (1. - y) + shaft * (0.18 + 0.2 * dust) * (0.5 + 0.5 * (y - cy) / (floor - cy))
    };
    let mut ui = Cv::new(k.lw, k.lh);
    dh_info(&mut ui, k, 3, "WINDOW.PIC", "EYE / 1 BIT / 8X8");
    dh_print(c, k, &tone, &ui);
}

/// 5 — DOOR: a doorway standing alone on a dark plain at dusk, light pouring out of it, and one figure before it
fn dh_door(c: &mut Cv, k: &Screen) {
    let wn = k.wn;
    let hz = 0.64;
    let (dx0, dw, dh0, dbot) = (wn * 0.5, 0.07, 0.27, 0.74);
    let tone = |x: f32, y: f32| -> f32 {
        let glow = (-(((x - dx0) / 1.4).powi(2) + (y - (dbot - dh0 * 0.5)).powi(2)).sqrt() / 0.08).exp();
        if y < hz {
            let dusk = 0.03 + 0.42 * ((y - 0.1) / (hz - 0.1)).max(0.).powf(1.8);
            return dusk + 0.35 * glow;
        }
        // the plain, darker toward you, the spill from the door widening over it
        let ground = 0.06 + 0.08 * dh_noise(x * 60., y * 200., 51) * (1. - (y - hz) / 0.3).max(0.);
        let spill = if y > dbot { let hw = dw * 0.5 + (y - dbot) * 0.7; if (x - dx0).abs() < hw { (1. - (y - dbot) / 0.3).max(0.) * dh_step(hw, hw * 0.85, (x - dx0).abs()) } else { 0. } } else { 0. };
        ground + spill * 0.85 + 0.45 * glow
    };
    let mut ui = Cv::new(k.lw, k.lh);
    let un = (k.lh as f32).min(k.lw as f32 * 0.5625);
    let (x0, y0, w, h) = ((dx0 - dw / 2.) * un, (dbot - dh0) * un, dw * un, dh0 * un);
    // the door: its frame in black, the opening blank white
    dh_px(&mut ui, k, x0 - 2., y0 - 2., w + 4., h + 2., false);
    dh_px(&mut ui, k, x0, y0, w, h, true);
    // the figure, black against the spill (head, shoulders, coat, legs) and its shadow running toward you
    let (fx, fy, z) = (dx0 * un + 5., 0.87 * un, 1.5);
    for (dx, dy, pw, ph) in [(-2., -26., 4., 4.), (-1., -22., 2., 1.), (-4., -21., 8., 12.), (-3., -9., 6., 3.), (-3., -6., 2., 6.), (1., -6., 2., 6.)] {
        dh_px(&mut ui, k, fx + dx * z, fy + dy * z, pw * z, ph * z, false);
    }
    for j in 0..34 { dh_px(&mut ui, k, fx - 3. + j as f32 * 0.3, fy + j as f32, 6. + j as f32 * 0.14, 1., false); }
    let mut r = Rng::new(505);
    for _ in 0..40 { let (x, y) = (r.f() * k.lw as f32, r.f() * un * 0.32); dh_px(&mut ui, k, x, y, 1., 1., true); }
    dh_info(&mut ui, k, 4, "DOOR.PIC", "DOOR / 1 BIT / 8X8");
    dh_print(c, k, &tone, &ui);
}

fn dither(c: &mut Cv, t: &Theme, i: usize) {
    let k = dh_screen(c, t);
    match i {
        0 => dh_beams(c, &k),
        1 => dh_clouds(c, &k),
        2 => dh_spiral(c, &k),
        3 => dh_window_art(c, &k),
        _ => dh_door(c, &k),
    }
}

/* ── Ghost Signal: five channels seen through a HUD's glass, in teal halftone with amber for the lights ───────────── */
// Each picture is a tone field screened into dots: teal on a 45° grid for everything, amber on a 15° grid for the
// lights. The lines (a wireframe, a lattice tower, raindrops, a figure) are drawn sharp over the dots, then the HUD:
// corner brackets, the channel, its name, a signal meter that weakens channel by channel, a lock-on reticle, scan lines.
// (The noise helpers are Dither's: dh_noise, dh_fbm, dh_step, dh_hash.)

struct Hud { bg: Rgb, ink: Rgb, amb: Rgb, red: Rgb, w: f32, h: f32, u: f32, wn: f32, display: String, mono: String }
fn gs_hud(c: &Cv, t: &Theme) -> Hud {
    let p = t.palette;
    let (w, h) = (c.w, c.h);
    let u = h.min(w * 0.5625);
    Hud { bg: p.bg, ink: p.ink, amb: p.accent, red: p.muted, w, h, u, wn: w / u, display: t.display.clone(), mono: t.mono.clone() }
}

/// a halftone screen: `tone` (0..1, in u units) sampled once per cell of a grid `g` px apart turned `ang`, each cell a
/// dot whose area follows the tone, laid over what is there in `ink`
fn gs_dots(c: &mut Cv, k: &Hud, g: f32, ang: f32, ink: Rgb, tone: &dyn Fn(f32, f32) -> f32) {
    let (w, h) = (c.px.width() as usize, c.px.height() as usize);
    let (cs, sn) = (ang.cos(), ang.sin());
    let (mut p0, mut p1, mut q0, mut q1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for (x, y) in [(0., 0.), (w as f32, 0.), (0., h as f32), (w as f32, h as f32)] {
        let (p, q) = (x * cs + y * sn, -x * sn + y * cs);
        p0 = p0.min(p); p1 = p1.max(p); q0 = q0.min(q); q1 = q1.max(q);
    }
    let (i0, j0) = ((p0 / g).floor() as i32 - 1, (q0 / g).floor() as i32 - 1);
    let (ni, nj) = (((p1 / g).ceil() as i32 + 2 - i0) as usize, ((q1 / g).ceil() as i32 + 2 - j0) as usize);
    let mut rad = vec![0f32; ni * nj];
    for j in 0..nj {
        for i in 0..ni {
            let (pc, qc) = (((i as i32 + i0) as f32 + 0.5) * g, ((j as i32 + j0) as f32 + 0.5) * g);
            let (x, y) = (pc * cs - qc * sn, pc * sn + qc * cs);
            if x < -g || y < -g || x > w as f32 + g || y > h as f32 + g { continue; }
            rad[j * ni + i] = g * 0.62 * tone(x / k.u, y / k.u).clamp(0., 1.).sqrt();
        }
    }
    let px = c.px.pixels_mut();
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let (p, q) = (fx * cs + fy * sn, -fx * sn + fy * cs);
            let (ci, cj) = ((p / g).floor() as i32 - i0, (q / g).floor() as i32 - j0);
            let mut cov = 0f32;
            for dj in -1..=1 {
                for di in -1..=1 {
                    let (ii, jj) = (ci + di, cj + dj);
                    if ii < 0 || jj < 0 || ii >= ni as i32 || jj >= nj as i32 { continue; }
                    let r = rad[jj as usize * ni + ii as usize];
                    if r <= 0. { continue; }
                    let (pc, qc) = (((ii + i0) as f32 + 0.5) * g, ((jj + j0) as f32 + 0.5) * g);
                    cov = cov.max((r - ((p - pc).powi(2) + (q - qc).powi(2)).sqrt() + 0.5).clamp(0., 1.));
                }
            }
            if cov > 0. {
                let o = px[y * w + x];
                let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * cov) as u8;
                px[y * w + x] = tiny_skia::PremultipliedColorU8::from_rgba(m(o.red(), ink.0), m(o.green(), ink.1), m(o.blue(), ink.2), 255).unwrap();
            }
        }
    }
}
/// the glass: every third line darkened, as a scanning display draws
fn gs_scan(c: &mut Cv) {
    let w = c.px.width() as usize;
    for (n, p) in c.px.pixels_mut().iter_mut().enumerate() {
        if (n / w) % 3 == 0 {
            *p = tiny_skia::PremultipliedColorU8::from_rgba((p.red() as f32 * 0.72) as u8, (p.green() as f32 * 0.72) as u8, (p.blue() as f32 * 0.72) as u8, 255).unwrap();
        }
    }
}
fn gs_path(pts: &[(f32, f32)], close: bool) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] { pb.line_to(p.0, p.1); }
    if close { pb.close(); }
    pb.finish()
}
/// a lock-on: four brackets round a square, a cross at its centre, its label and reading beside it in amber
fn gs_lock(c: &mut Cv, k: &Hud, x: f32, y: f32, r: f32, label: &str, read: &str) {
    let (lw, b) = ((k.u * 0.0012).max(1.), r * 0.35);
    let s = Cv::solid(col(k.amb, 0.95));
    for (sx, sy) in [(-1., -1.), (1., -1.), (-1., 1.), (1., 1.)] {
        let (cx, cy) = (x + sx * r, y + sy * r);
        c.line(cx, cy, cx - sx * b, cy, s.clone(), lw);
        c.line(cx, cy, cx, cy - sy * b, s.clone(), lw);
    }
    c.line(x - r * 0.18, y, x + r * 0.18, y, Cv::solid(col(k.amb, 0.7)), lw);
    c.line(x, y - r * 0.18, x, y + r * 0.18, Cv::solid(col(k.amb, 0.7)), lw);
    c.line(x + r, y - r, x + r * 1.35, y - r * 1.3, Cv::solid(col(k.amb, 0.7)), lw);
    let fs = k.u * 0.0135;
    c.text(label, &k.mono, fs, x + r * 1.4, y - r * 1.3 - fs * 0.3, col(k.amb, 0.95));
    c.text(read, &k.mono, fs * 0.85, x + r * 1.4, y - r * 1.3 + fs * 1.05, col(k.ink, 0.75));
}
/// the HUD over every channel: corner brackets, the channel and its name, the meter (five bars, one fewer lit on each
/// channel down the dial), a tick rule along the foot with the readings, and the glass
fn gs_frame(c: &mut Cv, k: &Hud, i: usize, name: &str, sub: &str, foot: &str) {
    let (w, h, u) = (k.w, k.h, k.u);
    let lw = (u * 0.0012).max(1.);
    let (m, b) = (u * 0.04, u * 0.05);
    let ink = |a: f32| Cv::solid(col(k.ink, a));
    for (x, y, sx, sy) in [(m, m, 1., 1.), (w - m, m, -1., 1.), (m, h - m, 1., -1.), (w - m, h - m, -1., -1.)] {
        c.line(x, y, x + sx * b, y, ink(0.85), lw * 1.6);
        c.line(x, y, x, y + sy * b, ink(0.85), lw * 1.6);
    }
    let (x0, fs) = (m + u * 0.016, u * 0.015);
    c.text(&format!("GHOST SIGNAL // CH-0{}", i + 1), &k.mono, fs, x0, m + u * 0.035, col(k.ink, 0.8));
    c.text(name, &k.display, fs * 3., x0, m + u * 0.085, col(k.ink, 0.95));
    c.text(sub, &k.mono, fs, x0, m + u * 0.112, col(k.amb, 0.9));
    // the meter, top right: five bars rising, lit as strong as the channel comes in
    let lit = 5 - i.min(4);
    for n in 0..5 {
        let (bw, bh) = (u * 0.008, u * (0.012 + n as f32 * 0.007));
        let x = w - m - u * 0.016 - (5 - n) as f32 * bw * 1.7;
        let s = if n < lit { Cv::solid(col(if lit <= 1 { k.red } else { k.ink }, 0.9)) } else { ink(0.18) };
        c.fill_rect(x, m + u * 0.045 - bh, bw, bh, s);
    }
    let tag = ["LOCKED", "LOCKED", "HOLDING", "DRIFTING", "LOST"][i.min(4)];
    let tw = crate::paint::measure(tag, &k.mono, fs);
    c.text(tag, &k.mono, fs, w - m - u * 0.016 - tw, m + u * 0.07, col(if i >= 4 { k.red } else { k.amb }, 0.9));
    // the foot: a rule of ticks between the lower brackets, the readings over its left end
    let (fy, fx0, fx1) = (h - m - u * 0.012, m + b + u * 0.02, w - m - b - u * 0.02);
    c.line(fx0, fy, fx1, fy, ink(0.35), lw);
    let mut x = fx0;
    let mut n = 0;
    while x <= fx1 { c.line(x, fy, x, fy - if n % 5 == 0 { u * 0.012 } else { u * 0.005 }, ink(if n % 5 == 0 { 0.55 } else { 0.3 }), lw); x += u * 0.012; n += 1; }
    c.text(foot, &k.mono, fs * 0.9, fx0, fy - u * 0.02, col(k.ink, 0.7));
}
fn gs_ground(c: &mut Cv, k: &Hud) { c.fill_all(Cv::solid(col(k.bg, 1.))); }
fn gs_glass(c: &mut Cv, k: &Hud) {
    gs_scan(c);
    c.vignette(k.bg, 0.75);
    c.cloud(k.w * 0.82, k.h * 0.12, k.u * 0.9, k.ink, 0.05);
}

/// the skyline: the far towers' tops and the near blocks' tops at `x` (u units); the tallest far tower carries the sign
fn gs_far(x: f32, wn: f32) -> f32 {
    let s = (x / 0.052).floor() as i32;
    let tall = (-((x - wn * 0.63) / 0.22).powi(2)).exp();
    0.66 - (0.08 + 0.22 * dh_hash(s, 3, 7)) * (0.7 + 0.6 * tall) - if s == ((wn * 0.63) / 0.052).floor() as i32 { 0.14 } else { 0. }
}
fn gs_near(x: f32) -> f32 { let s = (x / 0.09 + 0.37).floor() as i32; 0.86 - 0.2 * dh_hash(s, 9, 7) }
fn gs_window(x: f32, y: f32, seed: u32, p: f32) -> bool {
    let (wx, wy) = ((x / 0.011).floor() as i32, (y / 0.016).floor() as i32);
    let (fx, fy) = (x / 0.011 - wx as f32, y / 0.016 - wy as f32);
    fx > 0.25 && fx < 0.75 && fy > 0.3 && fy < 0.72 && dh_hash(wx, wy, seed) > p
}

/// CH-01 — CITY: the towers in the fog, the blocks in front with their windows lit, a sign down the tallest tower
fn gs_city(c: &mut Cv, k: &Hud) {
    let wn = k.wn;
    let sign = wn * 0.63;
    let teal = |x: f32, y: f32| -> f32 {
        let nt = gs_near(x);
        if y > nt { return 0.04 + if gs_window(x, y, 21, 0.78) { 0.55 } else { 0. }; }
        let ft = gs_far(x, wn);
        if y > ft {
            let fog = dh_step(0.3, 0.75, y);
            return 0.04 + 0.07 * fog + if gs_window(x * 1.6, y * 1.6, 13, 0.8) { 0.32 } else { 0. };
        }
        // the sky: dark overhead, the city's glow coming up behind the towers
        0.02 + 0.46 * (-(0.72 - y).max(0.) / 0.3).exp() + 0.07 * dh_fbm(x * 3., y * 9., 5) * (1. - y)
    };
    let amber = |x: f32, y: f32| -> f32 {
        if y > gs_near(x) { return if gs_window(x, y, 31, 0.93) { 0.9 } else { 0. }; }
        // the sign: a column of bars down the tower's face, and the glow it throws
        let (dx, top) = ((x - sign).abs(), gs_far(sign, wn) + 0.04);
        let on = dx < 0.011 && y > top && y < top + 0.2 && ((y - top) / 0.02).fract() < 0.72;
        if on { 0.95 } else { 0.35 * (-((dx / 0.05).powi(2) + ((y - top - 0.1) / 0.14).powi(2))).exp() * if y < gs_far(x, wn) { 1. } else { 0.4 } }
    };
    gs_ground(c, k);
    gs_dots(c, k, k.u * 0.0072, 0.785, k.ink, &teal);
    gs_dots(c, k, k.u * 0.0072, 0.262, k.amb, &amber);
    // aircraft lights on the tall towers
    let u = k.u;
    for xx in [wn * 0.12, wn * 0.36, wn * 0.63, wn * 0.88] {
        let y = gs_far(xx, wn);
        c.circle(xx * u, (y - 0.004) * u, u * 0.0028, Cv::solid(col(k.red, 0.9)));
        c.cloud(xx * u, (y - 0.004) * u, u * 0.012, k.red, 0.35);
    }
    gs_glass(c, k);
    gs_lock(c, k, sign * u, (gs_far(sign, wn) + 0.14) * u, u * 0.05, "TGT 07 // SIGN", "RANGE 1.24 KM");
    gs_frame(c, k, 0, "CITY", "NIGHT // FOG 62%", "LAT 35.4410  LON 139.6380  ALT 214 M");
}

/// CH-02 — RAIN: the city's lights out of focus beyond the glass, and the rain on it, beaded and running
fn gs_rain(c: &mut Cv, k: &Hud) {
    let (wn, u) = (k.wn, k.u);
    let mut r = Rng::new(202);
    let blobs: Vec<(f32, f32, f32, f32, bool)> = (0..34).map(|_| {
        let x = r.f() * wn; let y = 0.15 + r.f() * 0.8; let s = 0.025 + r.f().powi(2) * 0.09;
        (x, y, s, 0.35 + r.f() * 0.6, r.f() < 0.3)
    }).collect();
    let light = |x: f32, y: f32, amb: bool| -> f32 {
        blobs.iter().filter(|b| b.4 == amb).map(|b| {
            let d = ((x - b.0).powi(2) + (y - b.1).powi(2)).sqrt() / b.2;
            b.3 * (1. - dh_step(0.45, 1., d)) * (0.8 + 0.2 * d)
        }).sum::<f32>().min(1.)
    };
    let teal = |x: f32, y: f32| 0.03 + 0.08 * y + light(x, y, false) * 0.8;
    let amber = |x: f32, y: f32| light(x, y, true);
    gs_ground(c, k);
    gs_dots(c, k, u * 0.0085, 0.785, k.ink, &teal);
    gs_dots(c, k, u * 0.0085, 0.262, k.amb, &amber);
    gs_scan(c);
    // the drops: each darker within, lit along its foot where the light comes through it, a point of shine at its
    // brow; some have run, and left a trail
    let lw = (u * 0.001).max(1.);
    for n in 0..420 {
        let (x, y) = (r.f() * k.w, r.f() * k.h);
        let rr = u * (0.002 + r.f().powi(3) * 0.012);
        c.circle(x, y, rr, Cv::solid(col(k.bg, 0.45)));
        if let Some(a) = Cv::arc(x, y, rr * 0.86, rr * 0.86, 0.35, PI - 0.35) { c.stroke(&a, Cv::solid(col(k.ink, 0.75)), lw); }
        c.circle(x - rr * 0.35, y - rr * 0.4, (rr * 0.18).max(0.7), Cv::solid(col(Rgb(255, 255, 255), 0.7)));
        if n % 6 == 0 && rr > u * 0.005 {
            let len = u * (0.05 + r.f() * 0.25);
            let mut pts = vec![(x, y + rr)];
            let mut yy = y + rr;
            while yy < y + len { yy += u * 0.01; pts.push((x + (yy * 0.07).sin() * u * 0.002, yy)); }
            if let Some(p) = gs_path(&pts, false) { c.stroke(&p, Cv::solid(col(k.ink, 0.22)), rr * 0.5); }
        }
    }
    c.vignette(k.bg, 0.7);
    let b = blobs.iter().filter(|b| b.4).max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).unwrap();
    gs_lock(c, k, b.0 * u, b.1 * u, u * 0.045, "SRC 03 // LAMP", "LUX 0.42");
    gs_frame(c, k, 1, "RAIN", "PRECIP 87% // VIS 0.4 KM", "WIND 220° 14 KN  //  GLASS 4.1 °C");
}

/// CH-03 — TERRAIN: a ridge of hills in wireframe running off to a sun banded on the horizon
fn gs_terrain(c: &mut Cv, k: &Hud) {
    let (wn, u) = (k.wn, k.u);
    let hz = 0.6;
    let (sx, sy, sr) = (wn * 0.5, hz - 0.06, 0.17);
    let teal = |_x: f32, y: f32| -> f32 { if y > hz { 0.03 } else { 0.04 + 0.3 * (-(hz - y) / 0.16).exp() } };
    let amber = |x: f32, y: f32| -> f32 {
        let d = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
        if d < sr {
            // the lower half cut by bands that thicken toward the horizon
            let t = (y - (sy - sr * 0.1)) / (sr * 1.1);
            if t > 0. && ((y * 38.).fract() < t * 0.8) { return 0.; }
            0.55 + 0.4 * (1. - y / (sy + sr))
        } else { 0.4 * (-(d - sr) / 0.06).exp() }
    };
    gs_ground(c, k);
    gs_dots(c, k, u * 0.0072, 0.785, k.ink, &teal);
    gs_dots(c, k, u * 0.0072, 0.262, k.amb, &amber);
    let mut r = Rng::new(303);
    for _ in 0..160 { let (x, y) = (r.f() * k.w, r.f() * hz * 0.85 * u); c.circle(x, y, (u * 0.0012).max(0.8) * (0.5 + r.f()), Cv::solid(col(k.ink, 0.3 + r.f() * 0.5))); }
    // the land: a grid in the world, the valley down the middle, lifted by noise; drawn far to near, each quad first
    // blanked then outlined, so nearer hills hide farther ones
    let ht = |x: f32, z: f32| 0.04 + (x.abs().powf(1.5) * 0.32) * (0.5 + dh_fbm(x * 1.3 + 7., z * 0.9, 61)) + 0.05 * dh_noise(x * 4., z * 4., 62);
    let (cam, f) = (0.32, 0.62);
    let proj = |x: f32, z: f32| (sx * u + x / z * f * u, (hz + (cam - ht(x, z)) / z * f) * u);
    let cols = 56;
    let rows = 40;
    let zs: Vec<f32> = (0..=rows).map(|n| 0.22 * 1.075_f32.powi(n)).collect();
    let xs: Vec<f32> = (0..=cols).map(|n| (n as f32 / cols as f32 - 0.5) * 3.6).collect();
    let lw = (u * 0.0011).max(1.);
    for n in (0..rows).rev() {
        let (z0, z1) = (zs[n as usize], zs[n as usize + 1]);
        let fade = (1. - n as f32 / rows as f32).powf(0.8);
        for m in 0..cols {
            let q = [proj(xs[m], z0), proj(xs[m + 1], z0), proj(xs[m + 1], z1), proj(xs[m], z1)];
            if let Some(p) = gs_path(&q, true) {
                c.fill(&p, Cv::solid(col(k.bg, 1.)));
                c.stroke(&p, Cv::solid(col(k.ink, 0.12 + 0.75 * fade)), lw);
            }
        }
    }
    gs_glass(c, k);
    let (px, py) = proj(0.9, 1.4);
    gs_lock(c, k, px, py, u * 0.04, "WPT 12 // RIDGE", "ELEV 1,840 M");
    gs_frame(c, k, 2, "TERRAIN", "SCAN 3D // GRID 50 M", "HDG 271°  //  SPD 0.0  //  T+00:42:17");
}

/// CH-04 — TOWER: a lattice mast on the hill sending its rings out across a hazed sky, its beacon lit
fn gs_tower(c: &mut Cv, k: &Hud) {
    let (wn, u) = (k.wn, k.u);
    let (tx, ty, by) = (wn * 0.6, 0.15, 0.86);
    let hill = |x: f32| 0.86 + 0.04 * ((x - wn * 0.6) / 0.5).powi(2).min(1.) + 0.02 * (x * 7.).sin();
    let teal = |x: f32, y: f32| -> f32 {
        if y > hill(x) { return 0.02; }
        let d = ((x - tx).powi(2) + (y - ty).powi(2)).sqrt();
        let ring = dh_step(0.78, 0.97, (d * 12.).fract()) * (-d / 0.45).exp();
        let haze = 0.3 * (-(0.88 - y).max(0.) / 0.22).exp();
        let cloud = 0.14 * dh_step(0.5, 0.75, dh_fbm(x * 2.5, y * 7., 71)) * (1. - y);
        0.04 + haze + cloud + 0.22 * ring
    };
    let amber = |x: f32, y: f32| -> f32 {
        let d = ((x - tx).powi(2) + (y - ty + 0.02).powi(2)).sqrt();
        let city = if y > hill(x) && y < hill(x) + 0.08 && gs_window(x * 0.6, y * 1.4, 77, 0.9) { 0.8 } else { 0. };
        0.85 * (-d / 0.035).exp() + city
    };
    gs_ground(c, k);
    gs_dots(c, k, u * 0.0072, 0.785, k.ink, &teal);
    gs_dots(c, k, u * 0.0072, 0.262, k.amb, &amber);
    // the mast: two legs narrowing, a strut at each level and a cross in each bay, platforms, the aerial above;
    // black against the sky, a teal edge on the side the city light comes from
    let lvls = 16;
    let lv = |n: usize| -> (f32, f32) { let t = (n as f32 / lvls as f32).powf(1.25); (by - (by - ty) * t, 0.075 - (0.075 - 0.007) * t) };
    let mut segs: Vec<(f32, f32, f32, f32)> = vec![];
    for n in 0..lvls {
        let ((y0, w0), (y1, w1)) = (lv(n), lv(n + 1));
        segs.push((tx - w0, y0, tx - w1, y1)); segs.push((tx + w0, y0, tx + w1, y1));
        segs.push((tx - w1, y1, tx + w1, y1));
        segs.push((tx - w0, y0, tx + w1, y1)); segs.push((tx + w0, y0, tx - w1, y1));
    }
    segs.push((tx, ty, tx, ty - 0.07));
    for (n, ext) in [(5, 0.025), (10, 0.016)] { let (y, w) = lv(n); segs.push((tx - w - ext, y, tx + w + ext, y)); }
    for &(x0, y0, x1, y1) in &segs { c.line(x0 * u, y0 * u, x1 * u, y1 * u, Cv::solid(col(k.bg, 1.)), (u * 0.0052).max(2.5)); }
    for &(x0, y0, x1, y1) in &segs { c.line(x0 * u - 1., y0 * u, x1 * u - 1., y1 * u, Cv::solid(col(k.ink, 0.5)), (u * 0.001).max(0.8)); }
    for (n, d) in [(5, 0.03), (10, 0.02)] { let (y, w) = lv(n); for s in [-1., 1.] { c.fill_rect((tx + s * (w + d)) * u - u * 0.004, y * u - u * 0.01, u * 0.008, u * 0.012, Cv::solid(col(k.bg, 1.))); } }
    c.circle(tx * u, (ty - 0.07) * u, u * 0.004, Cv::solid(col(k.amb, 1.)));
    c.cloud(tx * u, (ty - 0.07) * u, u * 0.03, k.amb, 0.5);
    for n in [6, 11] { let (y, w) = lv(n); c.circle((tx - w) * u, y * u, u * 0.0025, Cv::solid(col(k.red, 0.9))); }
    gs_glass(c, k);
    gs_lock(c, k, tx * u, (ty - 0.07) * u, u * 0.035, "TX 04 // MAST", "118.4 MHZ  -62 DBM");
    gs_frame(c, k, 3, "TOWER", "CARRIER // 118.4 MHZ", "AZ 044°  EL 12°  //  SNR 9.6 DB");
}

/// CH-05 — GHOST: someone on a rooftop in the rain, hood up, looking out over the city; the HUD cannot name them
fn gs_ghost(c: &mut Cv, k: &Hud) {
    let (wn, u) = (k.wn, k.u);
    let roof = 0.84;
    let teal = |x: f32, y: f32| -> f32 {
        if y > roof { return 0.03; }
        let ft = gs_far(x * 0.8 + 0.3, wn) + 0.12;
        if y > ft { return 0.12 + 0.3 * dh_step(0.4, 0.84, y) + if gs_window(x * 2.2, y * 2.2, 91, 0.86) { 0.25 } else { 0. }; }
        0.05 + 0.4 * (-(0.8 - y).max(0.) / 0.3).exp() + 0.12 * dh_fbm(x * 2., y * 5., 93)
    };
    let amber = |x: f32, y: f32| -> f32 {
        if y > roof { return 0.; }
        let ft = gs_far(x * 0.8 + 0.3, wn) + 0.12;
        let neon = 0.55 * (-(((x - wn * 0.82) / 0.07).powi(2) + ((y - 0.5) / 0.05).powi(2))).exp();
        neon + if y > ft && gs_window(x * 2.2, y * 2.2, 97, 0.95) { 0.7 } else { 0. }
    };
    gs_ground(c, k);
    gs_dots(c, k, u * 0.0072, 0.785, k.ink, &teal);
    gs_dots(c, k, u * 0.0072, 0.262, k.amb, &amber);
    // the parapet, its edge catching the light
    c.fill_rect(0., roof * u, k.w, k.h, Cv::solid(col(k.bg, 1.)));
    c.line(0., roof * u, k.w, roof * u, Cv::solid(col(k.ink, 0.6)), (u * 0.0015).max(1.));
    // the figure, from behind: hood, shoulders, a long coat, a gap between the feet; rim-lit down its right side
    let (fx, fy, fh) = (wn * 0.32, roof, 0.38);
    let shape = [(0., -1.), (0.07, -0.98), (0.11, -0.92), (0.12, -0.82), (0.11, -0.76), (0.2, -0.72), (0.25, -0.66), (0.27, -0.5),
        (0.28, -0.3), (0.29, -0.14), (0.2, -0.12), (0.12, -0.12), (0.11, 0.), (0.03, 0.), (0.02, -0.1), (-0.02, -0.1), (-0.03, 0.),
        (-0.11, 0.), (-0.12, -0.12), (-0.2, -0.12), (-0.29, -0.14), (-0.28, -0.3), (-0.27, -0.5), (-0.25, -0.66), (-0.2, -0.72),
        (-0.11, -0.76), (-0.12, -0.82), (-0.11, -0.92), (-0.07, -0.98)];
    let pts: Vec<(f32, f32)> = shape.iter().map(|&(x, y)| ((fx + x * fh) * u, (fy + y * fh) * u)).collect();
    if let Some(p) = gs_path(&pts, true) { c.fill(&p, Cv::solid(col(k.bg, 1.))); }
    if let Some(p) = gs_path(&pts[0..11], false) { c.stroke(&p, Cv::solid(col(k.ink, 0.7)), (u * 0.0016).max(1.)); }
    // rain, slanting across everything
    let mut r = Rng::new(505);
    for _ in 0..700 {
        let (x, y, l) = (r.f() * k.w, r.f() * k.h, u * (0.01 + r.f() * 0.03));
        c.line(x, y, x - l * 0.25, y + l, Cv::solid(col(k.ink, 0.08 + r.f() * 0.18)), 1.);
    }
    gs_glass(c, k);
    gs_lock(c, k, fx * u, (fy - fh * 0.55) * u, u * 0.1, "SUBJECT // UNIDENTIFIED", "MATCH 0.00  //  NO RECORD");
    gs_frame(c, k, 4, "GHOST", "SIGNAL WEAK // SOURCE ?", "LAT --.----  LON ---.----  //  T+??:??:??");
}

fn ghost(c: &mut Cv, t: &Theme, i: usize) {
    let k = gs_hud(c, t);
    match i {
        0 => gs_city(c, &k),
        1 => gs_rain(c, &k),
        2 => gs_terrain(c, &k),
        3 => gs_tower(c, &k),
        _ => gs_ghost(c, &k),
    }
}

/* ── Y2K Blue: a 2001 desktop, five folders: glossy blue, chrome, pixel stars, a magenta highlight ──────────────── */
// Smooth things (skies, chrome, a CD's rainbow) are shaded pixel by pixel; pixel things (sparkles, sprites, the games
// screen) are drawn on a grid of big square pixels. Chrome reflects a made-up world: a white sky, a dark horizon band,
// a magenta ground glow, which is what made chrome look like chrome on those screens.

struct Y2 { bg: Rgb, blue: Rgb, mag: Rgb, peri: Rgb, pale: Rgb, w: f32, h: f32, u: f32, wn: f32, mono: String }
fn y2_k(c: &Cv, t: &Theme) -> Y2 {
    let p = t.palette;
    let u = c.h.min(c.w * 0.5625);
    Y2 { bg: p.bg, blue: p.surface, mag: p.accent, peri: p.accent2, pale: p.muted, w: c.w, h: c.h, u, wn: c.w / u,
        mono: t.mono.clone() }
}
/// every pixel through `f` (in u units); None leaves what is there
fn y2_shade(c: &mut Cv, k: &Y2, f: &dyn Fn(f32, f32) -> Option<Rgb>) {
    let (w, h) = (c.px.width() as usize, c.px.height() as usize);
    let px = c.px.pixels_mut();
    for y in 0..h {
        for x in 0..w {
            if let Some(o) = f((x as f32 + 0.5) / k.u, (y as f32 + 0.5) / k.u) {
                px[y * w + x] = tiny_skia::PremultipliedColorU8::from_rgba(o.0, o.1, o.2, 255).unwrap();
            }
        }
    }
}
fn y2_hsv(h: f32, s: f32, v: f32) -> Rgb {
    let h = h.rem_euclid(1.) * 6.;
    let (i, f) = (h.floor() as i32, h - h.floor());
    let (p, q, t) = (v * (1. - s), v * (1. - s * f), v * (1. - s * (1. - f)));
    let (r, g, b) = match i { 0 => (v, t, p), 1 => (q, v, p), 2 => (p, v, t), 3 => (p, q, v), 4 => (t, p, v), _ => (v, p, q) };
    Rgb((r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8)
}
/// what chrome reflects at height `e` (-1 straight up … 1 straight down): white sky, pale, blue to a dark horizon band,
/// then the magenta glow off the ground, deepening
fn y2_env(k: &Y2, e: f32) -> Rgb {
    if e < -0.02 { let t = (-e).min(1.); k.blue.mix(k.pale, (t * 2.2).min(1.)).mix(WHITE, ((t - 0.45) * 2.).clamp(0., 1.)) }
    else if e < 0.07 { k.bg.mix(BLACK, 0.3) }
    else { let t = ((e - 0.07) / 0.9).min(1.); k.mag.mix(k.pale, 0.25 * (1. - t * 3.).max(0.)).mix(k.bg, t.powf(0.7)) }
}
/// a pixel sparkle: a centre and four arms of `n` pixels, the arms' ends dimmer, at `z` px to a pixel
fn y2_spark(c: &mut Cv, x: f32, y: f32, n: i32, z: f32, col_: Rgb, a: f32) {
    let s = |c: &mut Cv, i: i32, j: i32, al: f32| c.fill_rect((x + i as f32 * z).round(), (y + j as f32 * z).round(), z, z, Cv::solid(col(col_, al)));
    s(c, 0, 0, a);
    for d in 1..=n { let al = a * (1. - (d - 1) as f32 / n as f32 * 0.6); s(c, d, 0, al); s(c, -d, 0, al); s(c, 0, d, al); s(c, 0, -d, al); }
    if n >= 3 { for (i, j) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] { s(c, i, j, a * 0.45); } }
}
/// a sprite from rows of characters: '#' ink, 'o' second colour, '.' nothing
fn y2_sprite(c: &mut Cv, rows: &[&str], x: f32, y: f32, z: f32, a: Rgb, b: Rgb) {
    for (j, r) in rows.iter().enumerate() {
        for (i, ch) in r.chars().enumerate() {
            let cc = match ch { '#' => a, 'o' => b, _ => continue };
            c.fill_rect(x + i as f32 * z, y + j as f32 * z, z, z, Cv::solid(col(cc, 1.)));
        }
    }
}
fn y2_rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let mut p = PathBuilder::new();
    p.move_to(x + r, y); p.line_to(x + w - r, y); p.quad_to(x + w, y, x + w, y + r); p.line_to(x + w, y + h - r); p.quad_to(x + w, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h); p.quad_to(x, y + h, x, y + h - r); p.line_to(x, y + r); p.quad_to(x, y, x + r, y); p.close();
    p.finish()
}
/// a glossy gel: the colour deepening downward, a white sheen over its top half, a pale rim
fn y2_gel(c: &mut Cv, x: f32, y: f32, w: f32, h: f32, r: f32, base: Rgb, k: &Y2) {
    if let Some(p) = y2_rrect(x, y, w, h, r) {
        c.fill(&p, Cv::linear(0., y, 0., y + h, &[(0., col(base.mix(WHITE, 0.25), 1.)), (0.55, col(base, 1.)), (1., col(base.mix(k.bg, 0.45), 1.))]));
        c.stroke(&p, Cv::solid(col(k.pale, 0.8)), (k.u * 0.0018).max(1.));
    }
    if let Some(p) = y2_rrect(x + w * 0.06, y + h * 0.05, w * 0.88, h * 0.45, r * 0.8) {
        c.fill(&p, Cv::linear(0., y, 0., y + h * 0.5, &[(0., col(WHITE, 0.75)), (1., col(WHITE, 0.08))]));
    }
}
fn y2_stars(c: &mut Cv, k: &Y2, seed: u64, n: usize, ymax: f32) {
    let mut r = Rng::new(seed);
    let z = (k.u / 270.).round().max(2.);
    for i in 0..n {
        let (x, y) = (r.f() * k.w, r.f() * k.h * ymax);
        let tint = if i % 5 == 0 { k.mag } else if i % 3 == 0 { k.pale } else { WHITE };
        match i % 9 { 0 => y2_spark(c, x, y, 3, z, tint, 0.95), 1 | 2 => y2_spark(c, x, y, 1, z, tint, 0.85), _ => c.fill_rect(x.round(), y.round(), z, z, Cv::solid(col(tint, 0.4 + r.f() * 0.5))) }
    }
}

/// HOME — the blue grid horizon: a floor of lines running off to a glowing horizon, a chrome ball floating over it,
/// the sky full of pixel stars
fn y2_home(c: &mut Cv, k: &Y2) {
    let (wn, u) = (k.wn, k.u);
    let hz = 0.62;
    let (bx, by, br) = (wn * 0.36, 0.38, 0.15);
    y2_shade(c, k, &|_x, y| {
        if y < hz {
            let t = y / hz;
            let sky = k.bg.mix(k.blue, t.powf(1.6)).mix(k.peri, ((t - 0.75) * 4.).clamp(0., 1.) * 0.8);
            return Some(sky.mix(WHITE, (-(hz - y) / 0.012).exp() * 0.9));
        }
        let t = ((y - hz) / (1. - hz)).min(1.);
        Some(k.blue.mix(k.bg, t.powf(0.5)).mix(k.mag, (-(y - hz) / 0.05).exp() * 0.35))
    });
    y2_stars(c, k, 11, 150, hz * 0.95);
    // the floor's lines: rows closing up toward the horizon, columns all running to one point on it
    let lw = (u * 0.0016).max(1.);
    for n in 1..40 {
        let z = 0.25 * 1.12_f32.powi(n);
        let y = (hz + 0.38 / z * 0.62) * u;
        if y > k.h { continue; }
        c.line(0., y, k.w, y, Cv::solid(col(k.pale, (0.85 - n as f32 * 0.02).max(0.15))), lw);
    }
    for n in -30..=30 {
        let x0 = wn * 0.5 * u + n as f32 * u * 0.11;
        let xb = wn * 0.5 * u + n as f32 * u * 0.11 * 9.;
        c.line(x0 + (wn * 0.5 * u - x0), hz * u, xb, k.h + u * 2., Cv::solid(col(k.pale, 0.55)), lw);
    }
    c.line(0., hz * u, k.w, hz * u, Cv::solid(col(WHITE, 0.9)), lw * 1.5);
    // the ball: chrome, the world wrapped round it, a hot highlight, its shadow on the floor
    c.cloud(bx * u, (hz + 0.06) * u, br * u * 1.1, k.bg, 0.6);
    y2_shade(c, k, &|x, y| {
        let (nx, ny) = ((x - bx) / br, (y - by) / br);
        let d2 = nx * nx + ny * ny;
        if d2 > 1. { return None; }
        let nz = (1. - d2).sqrt();
        let e = ny / (0.3 + 0.7 * nz) * 0.85 + 0.12;
        let spec = (-((nx + 0.38).powi(2) + (ny + 0.45).powi(2)) / 0.02).exp();
        let rim = (1. - nz).powi(3) * 0.5;
        Some(y2_env(k, e).mix(k.peri, rim).mix(WHITE, spec))
    });
    y2_spark(c, (bx - br * 0.38) * u, (by - br * 0.45) * u, 4, (u / 270.).round().max(2.), WHITE, 1.);
}

/// MUSIC — a CD turned to the light: silver, the rainbow running across its tracks in two bands, the clear ring and
/// the hole at its middle; notes in pixels drifting up past it
fn y2_music(c: &mut Cv, k: &Y2) {
    let (wn, u) = (k.wn, k.u);
    let (cx, cy, rx, ry) = (wn * 0.6, 0.52, 0.4, 0.36);
    y2_shade(c, k, &|x, y| {
        let g = k.bg.mix(k.blue, (1. - y).max(0.) * 0.8 + 0.2 * (x / wn));
        let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
        let rr = (dx * dx + dy * dy).sqrt();
        if rr > 1. || rr < 0.11 {
            // the halftone of the backdrop, and the disc's shadow
            let sh = if rr > 1. { (-(rr - 1.) / 0.06).exp() * 0.45 } else { 0. };
            return Some(g.mix(BLACK, sh));
        }
        let th = dy.atan2(dx);
        if rr < 0.15 { return Some(Rgb(0xD8, 0xDC, 0xE8)); }
        if rr < 0.33 {
            // the clear hub: the backdrop through it, lightened, a printed ring at its edge
            let ring = if (rr - 0.31).abs() < 0.006 || (rr - 0.17).abs() < 0.004 { 0.6 } else { 0. };
            return Some(g.mix(WHITE, 0.32 + ring));
        }
        let silver = Rgb(0xC9, 0xCF, 0xDC).mix(WHITE, (0.5 + 0.5 * (th * 2. + 0.8).cos()) * 0.55).mix(Rgb(0x8C, 0x94, 0xA8), ((th * 2. - 1.2).sin() * 0.5 + 0.5).powi(3) * 0.5);
        let band = (-((th - 0.75).sin().powi(2)) / 0.06).exp();
        let hue = rr * 2.2 + th * 0.18 + 0.55;
        let rain = y2_hsv(hue, 0.85, 1.);
        let grooves = 1. - 0.035 * ((rr * 900.).sin() * 0.5 + 0.5);
        let mut o = silver.mix(rain, band * 0.85);
        o = o.mix(BLACK, 1. - grooves);
        if rr > 0.975 { o = o.mix(Rgb(0x70, 0x78, 0x90), 0.6); }
        Some(o)
    });
    // pixel notes drifting up the left
    let z = (u / 270.).round().max(2.) * 3.;
    let note = ["..####", "..#..#", "..#..#", "..#..#", "###.##", "###.##", "###..."];
    let one = ["..#..", "..##.", "..#.#", "..#..", "###..", "###..", "###.."];
    for (i, (x, y)) in [(0.12, 0.3), (0.26, 0.55), (0.08, 0.72), (0.34, 0.2), (0.2, 0.86)].iter().enumerate() {
        let rows: &[&str] = if i % 2 == 0 { &note } else { &one };
        y2_sprite(c, rows, x * k.w + z * 0.5, y * k.h + z * 0.5, z, k.bg, k.bg);
        y2_sprite(c, rows, x * k.w, y * k.h, z, if i % 2 == 0 { k.pale } else { k.mag }, k.pale);
    }
    y2_stars(c, k, 21, 60, 1.);
    let z = (u / 270.).round().max(2.);
    y2_spark(c, (cx - rx * 0.45) * u, (cy - ry * 0.55) * u, 5, z, WHITE, 1.);
    y2_spark(c, (cx + rx * 0.5) * u, (cy + ry * 0.62) * u, 3, z, WHITE, 0.9);
}

/// PICS — a pixel collage: a checker of blues, glossy stickers with pixel icons on them, a little window with its own
/// picture, a banner, sparkles everywhere
fn y2_pics(c: &mut Cv, k: &Y2) {
    let u = k.u;
    let cell = u * 0.09;
    y2_shade(c, k, &|x, y| {
        let (i, j) = ((x * u / cell).floor() as i32, (y * u / cell).floor() as i32);
        Some(if (i + j) % 2 == 0 { k.blue } else { k.blue.mix(k.bg, 0.35) }.mix(k.bg, y * 0.35))
    });
    let z = (u / 270.).round().max(2.) * 2.;
    let heart = [".##.##.", "#######", "#######", "#######", ".#####.", "..###..", "...#..."];
    let star = ["....#....", "...###...", "...###...", "#########", ".#######.", "..#####..", "..##.##..", ".##...##.", ".#.....#."];
    let smile = ["..#####..", ".#.....#.", "#..#.#..#", "#..#.#..#", "#.......#", "#.#...#.#", "#..###..#", ".#.....#.", "..#####.."];
    let arrow = ["#......", "##.....", "#o#....", "#oo#...", "#ooo#..", "#oooo#.", "#oo####", "#.#o#..", "...#o#.", "....##."];
    let folder = ["###....", "#oo####", "#oooooo#", "#oooooo#", "#oooooo#", "########"];
    let mail = ["#########", "##.....##", "#.#...#.#", "#..#.#..#", "#...#...#", "#########"];
    let floppy = ["#######.", "#o###o##", "#o###o.#", "#ooooo.#", "#.....##", "#.ooo.##", "########"];
    // stickers: a gel tile with an icon on it
    let tiles: [(f32, f32, &[&str], Rgb, Rgb); 7] = [(0.06, 0.12, &heart, k.mag, WHITE), (0.2, 0.52, &star, k.peri, WHITE), (0.72, 0.1, &smile, Rgb(0xF2, 0xC2, 0x3A), k.bg),
        (0.85, 0.5, &folder, k.peri, Rgb(0xF2, 0xC2, 0x3A)), (0.44, 0.6, &mail, k.mag, WHITE), (0.08, 0.7, &floppy, k.blue, k.pale), (0.62, 0.38, &heart, k.peri, k.mag)];
    for (n, (x, y, s, base, ic)) in tiles.iter().enumerate() {
        let side = u * if n % 3 == 0 { 0.2 } else { 0.16 };
        let (x0, y0) = (x * k.w, y * k.h);
        c.cloud(x0 + side * 0.55, y0 + side * 0.6, side * 0.75, BLACK, 0.35);
        y2_gel(c, x0, y0, side, side, side * 0.22, *base, k);
        let (sw, sh) = (s[0].len() as f32, s.len() as f32);
        let zz = (side * 0.55 / sw.max(sh)).floor().max(2.);
        y2_sprite(c, s, x0 + (side - sw * zz) / 2., y0 + (side - sh * zz) / 2. + side * 0.04, zz, *ic, *ic);
    }
    // a little window: a gel title bar with its three buttons, a picture of hills and a sun inside
    let (wx, wy, ww, wh) = (k.w * 0.3, k.h * 0.14, u * 0.62, u * 0.42);
    c.cloud(wx + ww * 0.52, wy + wh * 0.56, ww * 0.6, BLACK, 0.35);
    if let Some(p) = y2_rrect(wx, wy, ww, wh, u * 0.012) { c.fill(&p, Cv::solid(col(k.pale, 1.))); c.stroke(&p, Cv::solid(col(WHITE, 0.9)), (u * 0.002).max(1.)); }
    y2_gel(c, wx, wy, ww, u * 0.05, u * 0.012, k.blue, k);
    c.text("my_pics.bmp", &k.mono, u * 0.034, wx + u * 0.016, wy + u * 0.037, col(WHITE, 1.));
    for (i, cc) in [k.mag, Rgb(0xF2, 0xC2, 0x3A), Rgb(0x5C, 0xD6, 0x7A)].iter().enumerate() { c.circle(wx + ww - u * (0.025 + i as f32 * 0.032), wy + u * 0.025, u * 0.011, Cv::solid(col(*cc, 1.))); }
    let (px0, py0, pw, ph) = (wx + u * 0.02, wy + u * 0.07, ww - u * 0.04, wh - u * 0.09);
    let pz = (u / 270.).round().max(2.) * 2.;
    let (cols_, rows_) = ((pw / pz) as i32, (ph / pz) as i32);
    for j in 0..rows_ {
        for i in 0..cols_ {
            let (fx, fy) = (i as f32 / cols_ as f32, j as f32 / rows_ as f32);
            let hill = 0.62 + 0.12 * (fx * 7.).sin() * (1. - fx) - 0.1 * (-((fx - 0.7) / 0.15).powi(2)).exp();
            let sun = ((fx - 0.72).powi(2) + ((fy - 0.3) * 0.75).powi(2)).sqrt() < 0.11;
            let cc = if fy > hill { if (i + j) % 2 == 0 && fy < hill + 0.06 { Rgb(0x3E, 0xA8, 0x5A) } else { Rgb(0x2E, 0x8A, 0x4A) } }
                else if sun { Rgb(0xF8, 0xD8, 0x4A) } else { k.peri.mix(WHITE, fy * 0.6) };
            c.fill_rect(px0 + i as f32 * pz, py0 + j as f32 * pz, pz, pz, Cv::solid(col(cc, 1.)));
        }
    }
    // the banner, and the cursor pointing at it
    let by = k.h * 0.88;
    let s = "~*~ welcome to my pics ~*~";
    let fs = u * 0.06;
    let tw = crate::paint::measure(s, &k.mono, fs);
    c.text(s, &k.mono, fs, (k.w - tw) / 2. + 3., by + 3., col(k.mag, 1.));
    c.text(s, &k.mono, fs, (k.w - tw) / 2., by, col(WHITE, 1.));
    y2_sprite(c, &arrow, k.w * 0.56, k.h * 0.6, z, BLACK, WHITE);
    y2_stars(c, k, 31, 90, 1.);
}

/// CHAT — a blob of liquid chrome, three drops of it pulling apart, the world bent round them; pixel speech bubbles
fn y2_chat(c: &mut Cv, k: &Y2) {
    let (wn, u) = (k.wn, k.u);
    let balls = [(wn * 0.5, 0.5, 0.2), (wn * 0.66, 0.38, 0.13), (wn * 0.38, 0.62, 0.12), (wn * 0.75, 0.6, 0.075)];
    y2_shade(c, k, &|x, y| {
        let bg = k.bg.mix(k.mag, (x / wn * 0.6 + y * 0.4).powf(2.) * 0.7).mix(k.blue, (1. - y) * 0.35 * (1. - x / wn));
        let (mut f, mut gx, mut gy) = (0f32, 0f32, 0f32);
        for &(bx, by, r) in &balls {
            let (dx, dy) = (x - bx, y - by);
            let d2 = (dx * dx + dy * dy).max(1e-5);
            f += r * r / d2;
            gx += -2. * r * r * dx / (d2 * d2);
            gy += -2. * r * r * dy / (d2 * d2);
        }
        if f < 1. { return Some(bg.mix(BLACK, ((f - 0.55) / 0.45).clamp(0., 1.) * 0.35)); }
        // the surface's normal from the field's slope: steep at the rim, flat on top
        // height z = H·sqrt(1 - 1/f): zero at the rim, rising fast, flat where the field is strong
        let hg = (1. - 1. / f).max(1e-4).sqrt();
        let q = 0.16 / (2. * f * f * hg);
        let (nx, ny) = (-gx * q, -gy * q);
        let l = (nx * nx + ny * ny + 1.).sqrt();
        let (nx, ny, nz) = (nx / l, ny / l, 1. / l);
        let e = ny / (0.3 + 0.7 * nz) * 0.85 + 0.12;
        let spec = (nx + 0.38).powi(2) + (ny + 0.45).powi(2);
        let o = y2_env(k, e).mix(k.peri, (1. - nz).powi(3) * 0.45).mix(WHITE, (-spec / 0.015).exp());
        Some(o.mix(BLACK, (1. - ((f - 1.) / 0.08).min(1.)) * 0.25))
    });
    // speech bubbles in pixels, with their words
    let z = (u / 270.).round().max(2.) * 2.;
    for (i, (x, y, s)) in [(0.07, 0.14, "hi!! :)"), (0.7, 0.1, "brb"), (0.1, 0.76, "lol"), (0.76, 0.8, "<3 <3")].iter().enumerate() {
        let (x0, y0) = (x * k.w, y * k.h);
        let bh = 7. * z * 2.;
        let bw = ((crate::paint::measure(s, &k.mono, bh * 0.62) + z * 5.) / z).ceil() * z;
        c.fill_rect(x0 + z, y0 + z, bw, bh, Cv::solid(col(BLACK, 0.4)));
        c.fill_rect(x0, y0, bw, bh, Cv::solid(col(WHITE, 1.)));
        c.fill_rect(x0 + z, y0 + z, bw - 2. * z, bh - 2. * z, Cv::solid(col(if i % 2 == 0 { WHITE } else { k.pale }, 1.)));
        for t in 0..3 { c.fill_rect(x0 + z * (3. + t as f32), y0 + bh + t as f32 * z, z * (3. - t as f32), z, Cv::solid(col(WHITE, 1.))); }
        let fs = bh * 0.62;
        c.text(s, &k.mono, fs, x0 + z * 2., y0 + bh * 0.72, col(if i % 2 == 0 { k.mag } else { k.bg }, 1.));
    }
    y2_stars(c, k, 41, 70, 1.);
}

/// GAMES — a screen of big pixels: dithered space, a ringed planet, a little ship with its trail, and the start line
fn y2_games(c: &mut Cv, k: &Y2) {
    let z = (k.u / 270.).round().max(2.) * 2.;
    let (lw, lh) = ((k.w / z).ceil() as u32, (k.h / z).ceil() as u32);
    let mut s = Cv::new(lw, lh);
    let (fw, fh) = (lw as f32, lh as f32);
    let un = fh.min(fw * 0.5625);
    // the sky in four tones, dithered into each other with the 8×8 order
    let tones = [k.bg, k.bg.mix(k.blue, 0.5), k.blue, k.peri];
    let (pcx, pcy, pr) = (fw * 0.66, fh * 0.46, un * 0.24);
    {
        let w = lw as usize;
        let px = s.px.pixels_mut();
        for y in 0..lh as usize {
            for x in 0..w {
                let (fx, fy) = (x as f32, y as f32);
                let (dx, dy) = ((fx - pcx) / pr, (fy - pcy) / pr);
                let d = (dx * dx + dy * dy).sqrt();
                let th = BAYER[y % 8][x % 8] as f32 / 64.;
                let c0 = if d < 1. {
                    // the planet: lit from the upper left, banded
                    let nz = (1. - d * d).sqrt();
                    let lit = ((-dx * 0.6 - dy * 0.6 + nz * 0.55) * 0.9 + 0.1).clamp(0., 1.);
                    let band = ((dy * 6. + 0.4 * (dx * 3.).sin()).sin() * 0.5 + 0.5) * 0.25;
                    let v = (lit * 3. + band) + th;
                    [k.bg, k.blue, k.peri, k.pale, WHITE][(v.floor() as usize).min(4)]
                } else {
                    let v = (fy / fh) * 1.8 + 0.4 * (-((fx - pcx).powi(2) + (fy - pcy).powi(2)).sqrt() / (pr * 2.)).exp() + th * 0.9;
                    tones[(v.floor() as usize).min(3)]
                };
                px[y * w + x] = tiny_skia::PremultipliedColorU8::from_rgba(c0.0, c0.1, c0.2, 255).unwrap();
            }
        }
    }
    // the ring: an ellipse of magenta and pale, in front below the planet, behind above it
    for n in 0..1400 {
        let a = n as f32 / 1400. * TAU;
        for (rk, cc) in [(1.55, k.mag), (1.7, k.pale), (1.85, k.mag)] {
            let (x, y) = (pcx + a.cos() * pr * rk, pcy + a.sin() * pr * rk * 0.28);
            let behind = a.sin() < 0. && ((x - pcx).powi(2) + (y - pcy).powi(2)).sqrt() < pr;
            if !behind { s.fill_rect(x.round(), y.round(), 1., 1., Cv::solid(col(cc, 1.))); }
        }
    }
    let mut r = Rng::new(51);
    for i in 0..160 {
        let (x, y) = ((r.f() * fw).round(), (r.f() * fh * 0.8).round());
        if ((x - pcx).powi(2) + (y - pcy).powi(2)).sqrt() < pr * 1.9 { continue; }
        let cc = if i % 4 == 0 { k.pale } else { WHITE };
        s.fill_rect(x, y, 1., 1., Cv::solid(col(cc, 1.)));
        if i % 11 == 0 { for (dx, dy) in [(1., 0.), (-1., 0.), (0., 1.), (0., -1.)] { s.fill_rect(x + dx, y + dy, 1., 1., Cv::solid(col(cc, 0.6))); } }
    }
    // the ship, going left to right, its trail behind it
    let ship = ["..##.....", ".#oo#....", "#oooo####", "#ooooooo#", "#oooo####", ".#oo#....", "..##....."];
    let (sx, sy) = ((fw * 0.18).round(), (fh * 0.66).round());
    for t in 0..26 { let a = 1. - t as f32 / 26.; s.fill_rect(sx - 2. - t as f32 * 2., sy + 6., 2., 2., Cv::solid(col(if t % 2 == 0 { k.mag } else { k.pale }, a))); }
    y2_sprite(&mut s, &ship, sx, sy, 2., WHITE, k.mag);
    c.draw_image(&s.px, 0., 0., lw as f32 * z, lh as f32 * z, false);
    // the start line, and the score
    let u = k.u;
    let line = "PRESS START";
    let fs = u * 0.07;
    let tw = crate::paint::measure(line, &k.mono, fs);
    c.text(line, &k.mono, fs, (k.w - tw) / 2. + z * 0.5, k.h * 0.88 + z * 0.5, col(k.mag, 1.));
    c.text(line, &k.mono, fs, (k.w - tw) / 2., k.h * 0.88, col(WHITE, 1.));
    c.text("1UP  002001", &k.mono, u * 0.04, u * 0.06, u * 0.08, col(WHITE, 0.9));
    c.text("HI  999999", &k.mono, u * 0.04, k.w - u * 0.06 - crate::paint::measure("HI  999999", &k.mono, u * 0.04), u * 0.08, col(k.pale, 0.9));
}

fn y2k(c: &mut Cv, t: &Theme, i: usize) {
    let k = y2_k(c, t);
    match i {
        0 => y2_home(c, &k),
        1 => y2_music(c, &k),
        2 => y2_pics(c, &k),
        3 => y2_chat(c, &k),
        _ => y2_games(c, &k),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_piece_draws_something() {
        for t in crate::theme::presets().into_iter().flatten() {
            for i in 0..super::PIECES {
                let px = super::piece(&t, i, 64, 36);
                assert!(px.pixels().iter().any(|p| p.alpha() > 0), "{} {i} is empty", t.id);
            }
        }
    }
}
