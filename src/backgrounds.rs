//! Procedural backgrounds: every theme looks complete before the user adds wallpapers, and nothing
//! copyrighted ships. Each draw works at any size (card faces and the 4K export use the same code).
use crate::paint::{col, rgba, Cv, Rng};
use crate::theme::{Rgb, Theme};
use std::f32::consts::{PI, TAU};
use tiny_skia::{BlendMode, PathBuilder, Transform};

fn h(s: &str) -> Rgb { Rgb::hex(s).unwrap() }

pub fn draw(c: &mut Cv, t: &Theme) {
    c.reset();
    if let Some(img) = crate::art::own(t.id, "walls").first().and_then(|p| crate::art::open(p)) {
        let px = crate::art::fill(&img, c.w as u32, c.h as u32);
        c.draw_image(&px, 0., 0., c.w, c.h, false);
        return;
    }
    match t.id {
        "fresco" => fresco(c),
        "relic" => relic(c),
        "inferno" => inferno(c, t),
        "schematic" => schematic(c, t),
        "marquee" => marquee(c, t),
        "dossier" => dossier(c, t),
        "xerox" => xerox(c, t),
        "etch" => etch(c),
        "dither" => dither(c),
        "ghost" => ghost(c, t),
        "y2k" => y2k(c, t),
        _ => ronin(c, t),
    }
    c.reset();
}

fn ronin(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(3);
    let u = w.min(hh);
    c.fill_all(Cv::radial(w * 0.62, hh * 0.45, w * 0.7, &[(0., col(h("#22252b"), 1.)), (1., col(t.palette.bg, 1.))]));
    // ensō: one brush circle built from many jittered strokes, thick at the start, dry at the end
    let (cx, cy, rad) = (w * 0.62, hh * 0.47, u * 0.3);
    for _ in 0..90 {
        let a = 0.05 + r.f() * 0.08;
        let a0 = -1.2 + r.f() * 0.15;
        let a1 = a0 + 5.6 - r.f() * 0.9;
        let off = (r.f() - 0.5) * u * 0.035;
        let lw = u * (0.004 + r.f() * 0.012);
        if let Some(p) = Cv::arc(cx, cy, rad + off, rad + off, a0, a1) { c.stroke(&p, Cv::solid(rgba(232, 226, 212, a)), lw); }
    }
    let s = u * 0.05;
    c.fill_rect(w * 0.86, hh * 0.76, s, s * 1.3, Cv::solid(col(t.palette.accent, 1.)));
    c.noise(0.08, h("#e8e2d4"), 1);
}

fn fresco(c: &mut Cv) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(11);
    let u = w.max(hh);
    c.fill_all(Cv::radial(w / 2., hh * 0.42, u * 0.6, &[(0., col(h("#fbf3df"), 1.)), (0.25, col(h("#a9cde8"), 1.)), (0.6, col(h("#5f97cf"), 1.)), (1., col(h("#2d4f78"), 1.))]));
    c.blend = BlendMode::Plus;
    let mut rot = 0f32;
    for _ in 0..26 {
        rot += TAU / 26. + r.f() * 0.05;
        let a = 0.025 + r.f() * 0.03;
        let mut b = PathBuilder::new();
        b.move_to(0., 0.); b.line_to(u, -u * 0.04); b.line_to(u, u * 0.04); b.close();
        c.tf = Transform::from_rotate(rot.to_degrees()).post_translate(w / 2., hh * 0.42);
        if let Some(p) = b.finish() { c.fill(&p, Cv::solid(rgba(255, 236, 190, a))); }
    }
    c.reset();
    for _ in 0..260 {
        let a = r.f() * TAU;
        let d = u * (0.2 + r.f() * 0.32);
        let rr = u * (0.03 + r.f() * 0.06);
        let cc = if r.f() > 0.3 { h("#fff6e6") } else { h("#8a7a6a") };
        c.cloud(w / 2. + a.cos() * d * 1.2, hh * 0.42 + a.sin() * d * 0.75, rr, cc, 0.35);
    }
    let gold = Cv::solid(col(h("#C9A24A"), 1.));
    if let Some(p) = Cv::arc(w / 2., hh * 0.45, w * 0.62, hh * 0.66, 0., TAU) { c.stroke(&p, gold.clone(), u * 0.012); }
    if let Some(p) = Cv::arc(w / 2., hh * 0.45, w * 0.6, hh * 0.64, 0., TAU) { c.stroke(&p, gold, u * 0.003); }
    c.vignette(h("#3B2A1E"), 0.85);
    c.noise(0.05, Rgb(0, 0, 0), 2);
}

fn relic(c: &mut Cv) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(5);
    let u = w.max(hh);
    c.fill_all(Cv::solid(col(h("#24170f"), 1.)));
    let oils = ["#5a3726", "#8c5a3e", "#3b2416", "#a8775a", "#1a0f09"];
    for _ in 0..420 {
        let cc = h(oils[(r.f() * 5.) as usize % 5]);
        c.alpha = 0.18 + r.f() * 0.25;
        let (x, y, rx, ry, rot) = (r.f() * w, r.f() * hh, u * (0.01 + r.f() * 0.05), u * (0.005 + r.f() * 0.02), r.f() * 3.);
        if let Some(p) = Cv::ellipse(x, y, rx, ry, rot) { c.fill(&p, Cv::solid(col(cc, 1.))); }
    }
    c.alpha = 1.;
    // the pasted window of sky
    let (cw, ch) = (w * 0.32, hh * 0.5);
    let base = Transform::from_rotate((-0.012f32).to_degrees()).post_translate(w / 2., hh / 2.);
    c.tf = base.pre_translate(u * 0.004, u * 0.008);
    c.alpha = 0.45;
    c.fill_rect(-cw / 2., -ch / 2., cw, ch, Cv::solid(rgba(0, 0, 0, 1.)));
    c.alpha = 1.;
    c.tf = base;
    c.fill_rect(-cw / 2., -ch / 2., cw, ch, Cv::linear(0., -ch / 2., 0., ch / 2., &[(0., col(h("#8EC1E0"), 1.)), (1., col(h("#d9ecf5"), 1.))]));
    if let Some(rect) = tiny_skia::Rect::from_xywh(-cw / 2., -ch / 2., cw, ch) { c.clip_path(&PathBuilder::from_rect(rect)); }
    for _ in 0..70 {
        let (x, y, rr) = ((r.f() - 0.5) * cw, (r.f() * 0.7 - 0.05) * ch - ch / 2. + ch * 0.1, u * (0.02 + r.f() * 0.04));
        c.cloud(x, y, rr, Rgb(255, 255, 255), 0.4);
    }
    c.reset();
    c.noise(0.22, h("#dcd2be"), 3);
    c.noise(0.18, Rgb(0, 0, 0), 4);
}

fn inferno(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(9);
    let u = w.max(hh);
    c.fill_all(Cv::linear(0., hh, 0., 0., &[(0., col(h("#5a1606"), 1.)), (0.45, col(h("#2a0d06"), 1.)), (1., col(h("#0d0604"), 1.))]));
    c.blend = BlendMode::Plus;
    for _ in 0..160 {
        let (x, y, len) = (r.f() * w, hh * (0.55 + r.f() * 0.5), u * (0.05 + r.f() * 0.2));
        let cc = if r.f() > 0.5 { t.palette.accent } else { t.palette.accent2 };
        c.alpha = 0.05 + r.f() * 0.12;
        let lw = u * (0.003 + r.f() * 0.012);
        let mut b = PathBuilder::new();
        b.move_to(x, y);
        let (a, bb, cx2, d, e) = ((r.f() - 0.5) * len, (r.f() - 0.5) * len, 0., (r.f() - 0.5) * len * 0.3, 0.);
        let _ = (cx2, e);
        b.cubic_to(x + a, y - len * 0.4, x + bb, y - len * 0.7, x + d, y - len);
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(cc, 1.)), lw); }
    }
    for _ in 0..220 {
        c.alpha = r.f() * 0.8;
        let (x, y, rr) = (r.f() * w, r.f() * hh * 0.9, u * 0.0012 * (1. + r.f() * 2.));
        c.circle(x, y, rr, Cv::solid(col(t.palette.accent2, 1.)));
    }
    c.reset();
    c.vignette(Rgb(0, 0, 0), 0.7);
    c.noise(0.06, Rgb(0, 0, 0), 5);
}

fn schematic(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(2);
    let u = w.max(hh);
    let split = w * 0.56;
    c.fill_all(Cv::solid(col(t.palette.bg, 1.)));
    let step = u / 160.;
    let major = u / 32.;
    let mut x = 0.;
    while x < w { let a = if x % major < step * 0.5 { 0.12 } else { 0.05 }; c.fill_rect(x, 0., 1., hh, Cv::solid(rgba(232, 230, 218, a))); x += step; }
    let mut y = 0.;
    while y < hh { let a = if y % major < step * 0.5 { 0.12 } else { 0.05 }; c.fill_rect(0., y, w, 1., Cv::solid(rgba(232, 230, 218, a))); y += step; }
    let line = Cv::solid(rgba(232, 230, 218, 0.55));
    let lw = (u * 0.0008).max(1.);
    let (cx, cy, rad) = (w * 0.25, hh * 0.42, u * 0.13);
    for k in 1..=7 { if let Some(p) = Cv::arc(cx, cy, rad * k as f32 / 7., rad * k as f32 / 7., 0., TAU) { c.stroke(&p, line.clone(), lw); } }
    for k in 0..36 {
        let a = k as f32 / 36. * TAU;
        c.line(cx + a.cos() * rad * 0.2, cy + a.sin() * rad * 0.2, cx + (a + 0.3).cos() * rad, cy + (a + 0.3).sin() * rad, line.clone(), lw);
    }
    for (s, x, y) in [("INTAKE", 0.05, 0.16), ("FAN STAGE", 0.38, 0.2), ("NACELLE", 0.06, 0.72), ("SECTOR A", 0.36, 0.78)] {
        c.text(s, "IBM Plex Mono", (u * 0.007).max(6.), w * x, hh * y, rgba(232, 230, 218, 0.8));
        c.line(w * x, hh * y + u * 0.004, cx, cy, line.clone(), lw);
    }
    // the painted half, behind a ragged edge
    let mut pts = Vec::new();
    let mut yy = 0.;
    while yy <= hh + 0.1 { pts.push((split + (r.f() - 0.5) * w * 0.06, yy)); yy += hh / 24.; }
    let mut b = PathBuilder::new();
    b.move_to(split, 0.);
    for p in &pts { b.line_to(p.0, p.1); }
    b.line_to(w, hh); b.line_to(w, 0.); b.close();
    if let Some(p) = b.finish() { c.clip_path(&p); }
    c.fill_all(Cv::linear(split, 0., w, hh, &[(0., col(h("#9fb7cf"), 1.)), (0.5, col(h("#6d5a48"), 1.)), (1., col(h("#2a1c14"), 1.))]));
    let paints = ["#a3322a", "#e9dcc4", "#3b2a1e", "#6f8fb0", "#c9a24a"];
    c.alpha = 0.25;
    for _ in 0..300 {
        let cc = h(paints[(r.f() * 5.) as usize % 5]);
        let (x, y, rx, rot) = (split + r.f() * (w - split), r.f() * hh, u * (0.005 + r.f() * 0.03), r.f() * 3.);
        if let Some(p) = Cv::ellipse(x, y, rx, u * 0.008, rot) { c.fill(&p, Cv::solid(col(cc, 1.))); }
    }
    c.reset();
    let mut b = PathBuilder::new();
    for (i, p) in pts.iter().enumerate() { if i == 0 { b.move_to(p.0, p.1) } else { b.line_to(p.0, p.1) } }
    if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(t.palette.accent, 1.)), u * 0.004); }
}

fn marquee(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    c.fill_all(Cv::solid(col(t.palette.bg, 1.)));
    c.headline("MARQUEE", &t.display, col(t.palette.accent, 1.), w * 0.04, hh * 0.12, w * 0.92);
    c.fill_rect(w * 0.04, hh * 0.8, w * 0.92, (hh * 0.004).max(2.), Cv::solid(col(t.palette.ink, 1.)));
    c.text("A RICE STUDIO PRODUCTION     WRITTEN AND DIRECTED BY YOU", &t.body, hh * 0.018, w * 0.04, hh * 0.85, col(t.palette.ink, 1.));
    c.noise(0.08, Rgb(0, 0, 0), 6);
}

fn dossier(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    c.fill_all(Cv::solid(col(t.palette.bg, 1.)));
    c.headline("CASE 07", &t.display, col(t.palette.accent, 1.), w * 0.06, hh * 0.06, w * 0.88);
    let red = col(t.palette.accent, 1.);
    for (s, x, y) in [("WRATH", 0.12, 0.7), ("PRIDE", 0.7, 0.66), ("ENVY", 0.42, 0.74), ("GREED", 0.1, 0.8), ("SLOTH", 0.66, 0.82)] {
        let (xx, yy) = (w * x, hh * y);
        c.text(s, "Courier Prime", hh * 0.013, xx, yy - hh * 0.006, red);
        c.line(xx, yy, xx + w * 0.16, yy, Cv::solid(red), (hh * 0.0012).max(1.));
    }
    c.noise(0.12, Rgb(0, 0, 0), 7);
}

fn xerox(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let u = w.max(hh);
    c.fill_all(Cv::solid(col(t.palette.bg, 1.)));
    let (cx, cy, step) = (w * 0.5, hh * 0.55, u * 0.008);
    let mut y = 0.;
    while y < hh {
        let mut x = 0.;
        while x < w {
            let d = ((x - cx).hypot(y - cy)) / (u * 0.22);
            if d <= 1.15 {
                let cc = if d < 0.55 { t.palette.accent } else { t.palette.ink };
                c.circle(x, y, step * 0.5 * (1. - (d - 0.55).abs()).max(0.), Cv::solid(col(cc, 1.)));
            }
            x += step;
        }
        y += step;
    }
    for k in 0..9 {
        let mut b = PathBuilder::new();
        let mut a = 0f32;
        while a <= 7. {
            let rr = u * (0.24 + k as f32 * 0.025) * (1. + (a * 5. + k as f32).sin() * 0.015);
            let (px, py) = (cx + a.cos() * rr, cy + a.sin() * rr * 0.9);
            if a == 0. { b.move_to(px, py) } else { b.line_to(px, py) }
            a += 0.05;
        }
        if let Some(p) = b.finish() { c.stroke(&p, Cv::solid(col(t.palette.ink, 1.)), u * 0.002); }
    }
    c.headline("XEROX", &t.display, col(t.palette.ink, 1.), w * 0.05, hh * 0.06, w * 0.5);
    c.noise(0.35, Rgb(0, 0, 0), 8);
    c.noise(0.12, h("#e9e4d8"), 9);
}

fn etch(c: &mut Cv) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(4);
    let u = w.max(hh);
    c.fill_all(Cv::solid(col(h("#0B0B0B"), 1.)));
    let (cx, cy) = (w * 0.55, hh * 0.38);
    for k in 0..140 {
        let rr = u * 0.06 + k as f32 * u * 0.0045;
        c.alpha = (0.9 - k as f32 / 160.).max(0.08);
        let rad = rr * (1. + (r.f() - 0.5) * 0.02);
        let a0 = r.f() * 6.;
        let a1 = r.f() * 6. + 3. + r.f() * 3.;
        let (a0, a1) = (a0, a0 + (a1 - a0).rem_euclid(TAU));
        if let Some(p) = Cv::arc(cx, cy, rad, rad, a0, a1) { c.stroke(&p, Cv::solid(col(h("#f2f2f2"), 1.)), (u * 0.0006).max(0.6)); }
    }
    c.alpha = 1.;
    c.circle(cx, cy, u * 0.06, Cv::solid(rgba(0, 0, 0, 1.)));
    for _ in 0..60 {
        let (a, d) = (r.f() * 7., r.f() * u * 0.055);
        c.fill_rect(cx + a.cos() * d, cy + a.sin() * d, 1.5, 1.5, Cv::solid(rgba(255, 255, 255, 1.)));
    }
}

fn dither(c: &mut Cv) {
    let (w, hh) = (c.w, c.h);
    const B: [f32; 16] = [0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.];
    let (sw, sh) = ((w / 4.).ceil() as u32, (hh / 4.).ceil() as u32);
    let mut small = Cv::new(sw, sh);
    for (i, p) in small.px.pixels_mut().iter_mut().enumerate() {
        let (x, y) = ((i as u32 % sw) as f32, (i as u32 / sw) as f32);
        let v = (1. - (((x - sw as f32 * 0.5) / sw as f32).hypot((y - sh as f32 * 0.62) / sh as f32)) * 2.2).max(0.).powf(1.6);
        let on = v * 16. > B[(y as usize % 4) * 4 + x as usize % 4] + 0.5;
        let g = if on { 255 } else { 0 };
        *p = tiny_skia::PremultipliedColorU8::from_rgba(g, g, g, 255).unwrap();
    }
    c.draw_image(&small.px, 0., 0., w, hh, false);
    for x in [0.3, 0.7] {
        c.fill_rect(w * x, hh * 0.08, (w * 0.0008).max(1.), hh * 0.8, Cv::solid(rgba(255, 255, 255, 1.)));
        c.circle(w * x, hh * 0.08, w * 0.004, Cv::solid(rgba(255, 255, 255, 1.)));
    }
}

fn ghost(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(8);
    let u = w.max(hh);
    c.fill_all(Cv::solid(col(t.palette.bg, 1.)));
    let (step, cx, cy) = (u * 0.007, w * 0.66, hh * 0.5);
    let mut y = 0.;
    while y < hh {
        let mut x = w * 0.3;
        while x < w {
            let d = ((x - cx) / (u * 0.2)).hypot((y - cy) / (u * 0.28));
            if d <= 1.2 {
                let cc = if d < 0.6 { h("#9CC9C0") } else { h("#1F4F4A") };
                c.circle(x, y, step * 0.48 * (1. - d * 0.7), Cv::solid(col(cc, 1.)));
            }
            x += step;
        }
        y += step;
    }
    let acc = col(t.palette.accent, 1.);
    let lw = (u * 0.0008).max(1.);
    for k in 0..3 {
        let (x, y, pw, ph) = (w * (0.08 + k as f32 * 0.12), hh * (0.25 + k as f32 * 0.18), w * 0.16, hh * 0.12);
        c.alpha = 0.8;
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, pw, ph) { c.stroke(&PathBuilder::from_rect(rect), Cv::solid(acc), lw); }
        c.alpha = 0.6;
        let lines = ["sys.link // ok", &format!("node 0x{:x}", (r.f() * 9e5) as u32), "sync 98.2%", "trace --deep", "ghost.signal", "auth: none"];
        for (l, s) in lines.iter().enumerate() {
            c.text(s, "Share Tech Mono", (u * 0.006).max(5.), x + u * 0.004, y + u * 0.008 + l as f32 * u * 0.0075, acc);
        }
    }
    c.alpha = 1.;
    let mut y = 0.;
    while y < hh { c.fill_rect(0., y, w, 1., Cv::solid(rgba(156, 201, 192, 0.035))); y += 3.; }
}

fn y2k(c: &mut Cv, t: &Theme) {
    let (w, hh) = (c.w, c.h);
    let mut r = Rng::new(6);
    let u = w.max(hh);
    c.fill_all(Cv::linear(0., 0., 0., hh, &[(0., col(h("#05093d"), 1.)), (1., col(h("#1A2BD6"), 1.))]));
    let grid = Cv::solid(rgba(108, 140, 255, 0.45));
    let lw = (u * 0.0008).max(1.);
    for i in -20..=20 { c.line(w / 2. + i as f32 * w * 0.01, hh * 0.6, w / 2. + i as f32 * w * 0.12, hh, grid.clone(), lw); }
    for k in 0..12 { let y = hh * 0.6 + hh * 0.4 * (k as f32 / 12.).powf(1.8); c.line(0., y, w, y, grid.clone(), lw); }
    let px = u * 0.008;
    let moon = ["#B6C7FF", "#6C8CFF", "#ffffff", "#8aa0ff"];
    for y in -14i32..=14 { for x in -14i32..=14 { if x * x + y * y < 196 {
        let cc = h(moon[(r.f() * 4.) as usize % 4]);
        c.fill_rect(w * 0.76 + x as f32 * px, hh * 0.26 + y as f32 * px, px, px, Cv::solid(col(cc, 1.)));
    } } }
    for cidx in 0..40 { for k in 0..8 {
        let a = 0.1 + r.f() * 0.3;
        let ch = char::from(33 + (r.f() * 90.) as u8);
        c.text(&ch.to_string(), "VT323", u * 0.01, cidx as f32 / 40. * w, hh * 0.05 + k as f32 * u * 0.012 + (cidx % 3) as f32 * 10., rgba(182, 199, 255, a));
    } }
    for _ in 0..40 {
        let cc = if r.f() > 0.7 { t.palette.accent } else { h("#6C8CFF") };
        let s = px * (1. + (r.f() * 3.).floor());
        c.fill_rect(r.f() * w, r.f() * hh * 0.6, s, s, Cv::solid(col(cc, 1.)));
    }
    let _ = PI;
}
