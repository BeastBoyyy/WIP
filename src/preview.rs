//! Native previews of what Apply will put on the desktop: the bar (drawn per style), the desktop clock and
//! the card faces for the gallery. The real bar is Zebar's HTML; these mirror its look, not its markup.
use crate::paint::{col, measure, rgba, Cv};
use crate::theme::{clock_text, now_hm, sanitize, Rgb, Theme};
use tiny_skia::{PathBuilder, Pixmap, Rect, Transform};

fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.).min(h / 2.);
    let mut b = PathBuilder::new();
    b.move_to(x + r, y);
    b.line_to(x + w - r, y); b.quad_to(x + w, y, x + w, y + r);
    b.line_to(x + w, y + h - r); b.quad_to(x + w, y + h, x + w - r, y + h);
    b.line_to(x + r, y + h); b.quad_to(x, y + h, x, y + h - r);
    b.line_to(x, y + r); b.quad_to(x, y, x + r, y);
    b.close();
    b.finish()
}

/// The bar as a transparent layer, `width` pixels wide, scaled from a 1920px-wide desktop.
pub fn bar(raw: &Theme, width: u32) -> Pixmap {
    let t = sanitize(raw);
    let p = t.palette;
    let s = width as f32 / 1920.;
    let bh = t.height as f32 * s;
    let mut c = Cv::new(width, (bh + 10. * s).ceil() as u32);
    let w = width as f32;
    let (hh, mm, date) = now_hm();
    let clock = clock_text(&t, hh, mm);
    let st = t.style;
    let fs = 13. * s;
    let ws_fs = match st { "credits" => 18., "dossier" => 19., "dither" => 20., "etch" => 17., "hud" => 14., "callout" => 13., _ => 16. } * s;
    let clock_fs = match st { "credits" => 26., "dossier" => 22., "dither" => 22., "xerox" => 20., "chrome" => 20., "islands" => 19., "collage" => 19., "hud" => 18., "callout" => 15., _ => 17. } * s;
    let (mut x0, mut x1) = (10. * s, w - 10. * s);
    // backgrounds
    match st {
        "islands" => {}
        "cartouche" => {
            let bw = w * 0.72;
            x0 = (w - bw) / 2. + 18. * s; x1 = (w + bw) / 2. - 18. * s;
            if let Some(pa) = rrect((w - bw) / 2., 3. * s, bw, bh - 4. * s, bh) {
                c.fill(&pa, Cv::linear(0., 0., 0., bh, &[(0., col(p.surface, 1.)), (1., col(p.surface.mix(p.accent, 0.2), 1.))]));
                c.stroke(&pa, Cv::solid(col(p.accent, 1.)), 2. * s);
            }
            if let Some(pa) = rrect((w - bw) / 2. + 5. * s, 8. * s, bw - 10. * s, bh - 14. * s, bh) { c.stroke(&pa, Cv::solid(col(p.accent, 1.)), 1. * s); }
        }
        "collage" => {
            let mut b = PathBuilder::new();
            b.move_to(0., 0.); b.line_to(w, 0.);
            let mut r = crate::paint::Rng::new(7);
            let mut x = 100.;
            while x >= 0. { b.line_to(w * x / 100., bh * (0.84 + r.f() * 0.16)); x -= 2.5; }
            b.close();
            if let Some(pa) = b.finish() { c.fill(&pa, Cv::solid(col(p.surface, 1.))); }
        }
        "ember" => {
            c.fill_rect(0., 0., w, bh, Cv::linear(0., 0., 0., bh, &[(0., col(Rgb(14, 7, 4), 1.)), (1., col(p.surface, 1.))]));
            c.fill_rect(0., bh - 2. * s, w, 2. * s, Cv::linear(0., 0., w, 0., &[(0., col(p.accent, 0.)), (0.3, col(p.accent, 1.)), (0.5, col(p.accent2, 1.)), (0.7, col(p.accent, 1.)), (1., col(p.accent, 0.))]));
        }
        "callout" => {
            c.fill_rect(0., 0., w, bh, Cv::solid(col(p.bg, 1.)));
            let mut x = 0.;
            while x < w { c.fill_rect(x, 0., 1., bh, Cv::solid(col(p.ink, 0.08))); x += 12. * s; }
            let mut y = 0.;
            while y < bh { c.fill_rect(0., y, w, 1., Cv::solid(col(p.ink, 0.08))); y += 12. * s; }
            c.fill_rect(0., bh - 1., w, 1., Cv::solid(col(p.ink, 1.)));
        }
        "credits" => { c.fill_rect(0., 0., w, bh, Cv::solid(col(p.surface, 1.))); c.fill_rect(0., bh - 3. * s, w, 3. * s, Cv::solid(col(p.ink, 1.))); }
        "dossier" => { c.fill_rect(0., 0., w, bh, Cv::solid(col(p.surface, 1.))); c.noise(0.08, Rgb(0, 0, 0), 11); c.fill_rect(0., bh - 1., w, 1., Cv::solid(col(p.accent, 1.))); }
        "xerox" => {
            c.fill_rect(0., 0., w, bh, Cv::solid(col(p.surface, 1.)));
            c.noise(0.25, Rgb(0, 0, 0), 12);
            let mut x = 3. * s;
            while x < w { c.circle(x, bh - 3. * s, 1.1 * s, Cv::solid(col(p.ink, 1.))); x += 6. * s; }
        }
        "etch" => {
            c.fill_rect(0., 0., w, bh, Cv::solid(col(p.bg, 1.)));
            let mut x = -bh;
            while x < w { c.line(x, bh, x + bh, 0., Cv::solid(rgba(255, 255, 255, 0.07)), 1.); x += 4. * s; }
            c.fill_rect(0., bh - 1., w, 1., Cv::solid(col(p.ink, 1.)));
            c.cloud(w / 2., bh / 2., 45. * s, Rgb(255, 255, 255), 0.35);
        }
        "dither" => {
            c.fill_rect(0., 0., w, bh, Cv::solid(rgba(0, 0, 0, 1.)));
            let k = (2. * s).max(1.);
            let mut x = 0.;
            let mut i = 0;
            while x < w { for j in 0..2 { if (i + j) % 2 == 0 { c.fill_rect(x, bh - 4. * s + j as f32 * k, k, k, Cv::solid(rgba(255, 255, 255, 1.))); } } x += k; i += 1; }
        }
        "hud" => {
            let mut b = PathBuilder::new();
            b.move_to(0., 0.); b.line_to(w, 0.); b.line_to(w, bh * 0.62); b.line_to(w - 16. * s, bh); b.line_to(16. * s, bh); b.line_to(0., bh * 0.62); b.close();
            if let Some(pa) = b.finish() { c.fill(&pa, Cv::linear(0., 0., w, 0., &[(0., col(p.surface, 0.92)), (1., col(p.bg, 0.75))])); }
            c.fill_rect(0., 0., w, 1., Cv::solid(col(p.accent2, 1.)));
            let mut y = 0.;
            while y < bh * 0.9 { c.fill_rect(16. * s, y, w - 32. * s, 1., Cv::solid(col(p.ink, 0.09))); y += 3. * s; }
        }
        "chrome" => {
            c.fill_rect(0., 0., w, bh, Cv::linear(0., 0., 0., bh, &[(0., col(Rgb(0x5b, 0x86, 0xff), 1.)), (0.46, col(p.surface, 1.)), (0.54, col(Rgb(0x0f, 0x1d, 0xa8), 1.)), (1., col(Rgb(0x2a, 0x46, 0xf0), 1.))]));
            c.fill_rect(0., 0., w, 1., Cv::solid(rgba(255, 255, 255, 0.65)));
            c.fill_rect(0., bh - 2. * s, w, 2. * s, Cv::solid(col(Rgb(8, 16, 90), 1.)));
        }
        _ => {}
    }
    let mid = bh / 2.;
    let ink = col(if st == "dither" { Rgb(255, 255, 255) } else { p.ink }, 1.);
    let muted = col(if st == "dither" { Rgb(0x77, 0x77, 0x77) } else { p.muted }, 1.);
    // islands behind groups (Vagabond, Y2K)
    let pad = 14. * s;
    let ws_w: Vec<f32> = t.ws.iter().map(|l| measure(l, &t.display, ws_fs).max(14. * s) + 12. * s).collect();
    let left_w = ws_w.iter().sum::<f32>() + 4. * s * (t.ws.len() as f32 - 1.) + pad * 2.;
    let date_w = measure(&date, &t.body, fs) + pad * 2.;
    let stats = ["12%", "41%", "87%"];
    let stats_w: f32 = stats.iter().map(|x| measure(x, &t.body, fs) + 14. * s).sum::<f32>() + measure(&clock, &t.display, clock_fs) + pad * 2.;
    let island = |c: &mut Cv, x: f32, wd: f32| {
        let (fill, line, r) = match st { "chrome" => (rgba(0, 0, 40, 0.35), rgba(255, 255, 255, 0.35), 4. * s), _ => (col(p.surface, 0.9), col(p.ink, 0.09), 12. * s) };
        if let Some(pa) = rrect(x, 5. * s, wd, bh - 10. * s, r) { c.fill(&pa, Cv::solid(fill)); c.stroke(&pa, Cv::solid(line), 1.); }
    };
    if matches!(st, "islands" | "chrome") {
        island(&mut c, x0, left_w);
        island(&mut c, w / 2. - date_w / 2., date_w);
        island(&mut c, x1 - stats_w, stats_w);
    }
    // workspaces
    let mut x = x0 + pad;
    for (i, l) in t.ws.iter().enumerate() {
        let iw = ws_w[i];
        let on = i == 0;
        let occ = i == 0 || i == 2;
        let bx = (x, mid - 13. * s, iw, 26. * s);
        let mut color = if on { col(p.accent, 1.) } else if occ { ink } else { muted };
        let mut rot = 0f32;
        match (st, on) {
            ("islands", true) => { color = col(p.ink, 1.); rot = -4.; }
            ("collage", true) => { color = col(p.bg, 1.); rot = -2.5; }
            ("xerox", true) => { color = col(p.surface, 1.); }
            ("dither", true) => { color = rgba(0, 0, 0, 1.); }
            ("cartouche", true) => { color = col(p.muted, 1.); }
            ("ember", true) => { color = col(p.accent2, 1.); c.cloud(bx.0 + iw / 2., mid, iw * 0.7, p.accent, 0.45); }
            _ => {}
        }
        if on {
            c.tf = Transform::from_rotate_at(rot, bx.0 + iw / 2., mid);
            let fill = match st {
                "islands" => Some(col(p.accent, 1.)),
                "collage" => Some(col(Rgb(0x8E, 0xC1, 0xE0), 1.)),
                "xerox" => Some(col(p.ink, 1.)),
                "dither" => Some(rgba(255, 255, 255, 1.)),
                "chrome" => Some(col(p.accent, 1.)),
                _ => None,
            };
            if let (Some(f), Some(pa)) = (fill, rrect(bx.0, bx.1, bx.2, bx.3, if st == "chrome" { 3. * s } else { 4. * s })) { c.fill(&pa, Cv::solid(f)); }
            match st {
                "cartouche" => c.fill_rect(bx.0 + 4. * s, mid + 9. * s, iw - 8. * s, 2. * s, Cv::solid(col(p.accent, 1.))),
                "callout" => if let Some(r) = Rect::from_xywh(bx.0, bx.1 + 3. * s, iw, bx.3 - 4. * s) { c.stroke(&PathBuilder::from_rect(r), Cv::solid(col(p.accent, 1.)), 1.) },
                "dossier" => c.fill_rect(bx.0, mid + 11. * s, iw, 2. * s, Cv::solid(col(p.accent, 1.))),
                "etch" => if let Some(pa) = Cv::arc(bx.0 + iw / 2., mid, iw / 2., 13. * s, 0., 7.) { c.stroke(&pa, Cv::solid(ink), 1.) },
                "dither" => c.fill_rect(bx.0 + iw / 2., bh, 1., 6. * s, Cv::solid(rgba(255, 255, 255, 1.))),
                _ => {}
            }
        } else if st == "chrome" {
            if let Some(pa) = rrect(bx.0, bx.1, bx.2, bx.3, 3. * s) { c.fill(&pa, Cv::linear(0., bx.1, 0., bx.1 + bx.3, &[(0., col(Rgb(0xa9, 0xc1, 0xff), 1.)), (0.5, col(Rgb(0x4a, 0x6c, 0xf0), 1.)), (1., col(Rgb(0x33, 0x55, 0xe0), 1.))])); }
        }
        let label = if st == "hud" && on { format!("[{l}]") } else { l.clone() };
        let lw = measure(&label, &t.display, ws_fs);
        if st == "chrome" { color = rgba(255, 255, 255, 1.); }
        c.text(&label, &t.display, ws_fs, bx.0 + (iw - lw) / 2., mid + ws_fs * 0.35, color);
        c.tf = Transform::identity();
        x += iw + 4. * s;
    }
    // centre: date
    let date_col = match st { "cartouche" | "callout" => col(p.accent, 1.), "collage" => col(p.accent2, 1.), _ => ink };
    let dw = measure(&date, &t.body, fs);
    c.text(&date, &t.body, fs, w / 2. - dw / 2., mid + fs * 0.35, date_col);
    if st == "credits" { c.cloud(w / 2. - dw / 2. - 16. * s, mid, 9. * s, p.accent, 1.); }
    // right: stats and clock
    let cw = measure(&clock, &t.display, clock_fs);
    let mut rx = x1 - pad - cw;
    let clock_col = match st { "credits" => rgba(255, 255, 255, 1.), "ember" => col(p.accent2, 1.), "callout" | "dossier" => col(p.accent, 1.), "chrome" => col(Rgb(0xcf, 0xe0, 0xff), 1.), _ => ink };
    if st == "credits" { c.fill_rect(rx - 16. * s, 0., cw + 32. * s + pad, bh - 3. * s, Cv::solid(col(p.accent, 1.))); rx -= 0.; }
    if st == "ember" { c.cloud(rx + cw / 2., mid, cw * 0.6, p.accent, 0.4); }
    c.text(&clock, &t.display, clock_fs, rx, mid + clock_fs * 0.35, clock_col);
    let stat_col = if st == "hud" { col(p.accent, 1.) } else { ink };
    rx -= if st == "credits" { 30. * s } else { 14. * s };
    for v in stats.iter().rev() {
        let vw = measure(v, &t.body, fs);
        rx -= vw;
        c.text(v, &t.body, fs, rx, mid + fs * 0.35, stat_col);
        rx -= 14. * s;
    }
    c.px
}

/// The desktop clock as a transparent layer sized for a `width`-wide desktop.
pub fn clock(t: &Theme, width: u32, over: Rgb) -> Pixmap {
    let s = width as f32 / 1920.;
    let (h, m, _) = now_hm();
    let txt = clock_text(t, h, m);
    let size = match t.clock { "headline" => 230., "roman" => 110., "digital" | "brush" => 170., _ => 150. } * s;
    let tw = measure(&txt, &t.display, size);
    let mut c = Cv::new((tw + 40. * s).ceil() as u32, (size * 1.3).ceil() as u32);
    // the clock takes whichever theme colour reads best on what's behind it
    let p = t.palette;
    let ink = *[p.ink, p.surface, p.bg, p.accent].iter().max_by(|a, b| (a.luma() - over.luma()).abs().total_cmp(&(b.luma() - over.luma()).abs())).unwrap();
    c.alpha = 0.25;
    c.text(&txt, &t.display, size, 20. * s, size * 1.02 + 4. * s, rgba(0, 0, 0, 1.));
    c.alpha = 1.;
    c.text(&txt, &t.display, size, 20. * s, size * 1.02, col(ink, 1.));
    c.px
}

/// average colour of a region (to pick the clock ink)
pub fn region_luma(px: &Pixmap, x: f32, y: f32, w: f32, h: f32) -> Rgb {
    let (x0, y0) = ((x * px.width() as f32) as u32, (y * px.height() as f32) as u32);
    let (x1, y1) = (((x + w) * px.width() as f32) as u32, ((y + h) * px.height() as f32) as u32);
    let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
    for yy in (y0..y1.min(px.height())).step_by(4) {
        for xx in (x0..x1.min(px.width())).step_by(4) {
            let q = px.pixel(xx, yy).unwrap();
            r += q.red() as u64; g += q.green() as u64; b += q.blue() as u64; n += 1;
        }
    }
    let n = n.max(1);
    Rgb((r / n) as u8, (g / n) as u8, (b / n) as u8)
}

/// The three layers of a desktop preview: wallpaper, bar, clock (clock positioned bottom-left).
pub struct Desk { pub wall: Pixmap, pub bar: Pixmap, pub clock: Pixmap }
pub fn desk(t: &Theme, wall: Pixmap) -> Desk {
    let w = wall.width();
    let under = region_luma(&wall, 0., 0.72, 0.35, 0.24);
    Desk { bar: bar(t, w), clock: clock(t, w, under), wall }
}

/// A gallery card face: the full desktop flattened (the name is set by the UI beside it).
pub fn card(t: &Theme, w: u32, h: u32) -> Pixmap {
    let mut c = Cv::new(w, h);
    crate::backgrounds::draw(&mut c, t);
    let b = bar(t, w);
    c.draw_image(&b, 0., 0., w as f32, b.height() as f32, true);
    let under = region_luma(&c.px, 0., 0.72, 0.35, 0.24);
    let k = clock(t, w, under);
    c.draw_image(&k, w as f32 * 0.036, h as f32 * 0.945 - k.height() as f32, k.width() as f32, k.height() as f32, true);
    c.px
}
