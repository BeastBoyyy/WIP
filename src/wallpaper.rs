//! Composes the desktop wallpaper: the user's own images (or the procedural background), laid out for a
//! landscape screen, then the theme's treatment so any picture matches the theme.
use crate::backgrounds;
use crate::paint::{blurred, col, Cv};
use crate::theme::{Rgb, Theme};
use tiny_skia::{BlendMode, PathBuilder, Pixmap, PremultipliedColorU8, Rect};

pub const LAYOUTS: [(&str, &str); 4] = [("fill", "Fill"), ("blur", "Blur fill"), ("poster", "Poster frame"), ("triptych", "Triptych")];

pub struct Shot<'a> { pub w: u32, pub h: u32, pub theme: &'a Theme, pub images: &'a [Pixmap], pub layout: &'a str, pub treatment: &'a str, pub strength: f32 }

fn cover(c: &mut Cv, img: &Pixmap, x: f32, y: f32, w: f32, h: f32) {
    let s = (w / img.width() as f32).max(h / img.height() as f32);
    let (iw, ih) = (img.width() as f32 * s, img.height() as f32 * s);
    if let Some(r) = Rect::from_xywh(x, y, w, h) { c.clip_path(&PathBuilder::from_rect(r)); }
    c.draw_image(img, x + (w - iw) / 2., y + (h - ih) / 2., iw, ih, true);
    c.clip = None;
}
fn contain(c: &mut Cv, img: &Pixmap, cx: f32, cy: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let s = (w / img.width() as f32).min(h / img.height() as f32);
    let (iw, ih) = (img.width() as f32 * s, img.height() as f32 * s);
    c.draw_image(img, cx - iw / 2., cy - ih / 2., iw, ih, true);
    (cx - iw / 2., cy - ih / 2., iw, ih)
}

fn lum(p: PremultipliedColorU8) -> f32 { 0.299 * p.red() as f32 + 0.587 * p.green() as f32 + 0.114 * p.blue() as f32 }
fn put(c: Rgb) -> PremultipliedColorU8 { PremultipliedColorU8::from_rgba(c.0, c.1, c.2, 255).unwrap() }

fn treat(px: &mut Pixmap, kind: &str, t: &Theme) {
    let p = t.palette;
    let dark = p.bg.luma() < p.ink.luma();
    let (lo, hi) = if dark { (p.bg, p.ink) } else { (p.ink, p.bg) };
    let (w, h) = (px.width() as usize, px.height() as usize);
    let mut seed = 0x9E37_79B9u32;
    let mut rnd = move || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; (seed & 0xffff) as f32 / 65535. };
    let each = |px: &mut Pixmap, f: &mut dyn FnMut(PremultipliedColorU8, usize, usize) -> PremultipliedColorU8| {
        for (i, q) in px.pixels_mut().iter_mut().enumerate() { *q = f(*q, i % w, i / w); }
    };
    match kind {
        "grain" => each(px, &mut |q, _, _| {
            let n = (rnd() - 0.5) * 46.;
            let c = |v: u8, o: f32| (v as f32 + n + o).clamp(0., 255.) as u8;
            PremultipliedColorU8::from_rgba(c(q.red(), 8.), c(q.green(), 3.), c(q.blue(), -6.), 255).unwrap()
        }),
        "bw" => each(px, &mut |q, _, _| { let v = ((lum(q) - 128.) * 1.6 + 128.).clamp(0., 255.) as u8; put(Rgb(v, v, v)) }),
        "duotone" => each(px, &mut |q, _, _| put(lo.mix(p.accent2, lum(q) / 255.))),
        "inkwash" => each(px, &mut |q, _, _| put(lo.mix(hi, (lum(q) / 255.).powf(1.25)))),
        "xerox" => each(px, &mut |q, _, _| put(if lum(q) + (rnd() - 0.5) * 70. > 118. { hi } else { lo })),
        "engrave" => each(px, &mut |q, x, y| {
            let s = (y as f32 * std::f32::consts::PI / 3. + (x as f32 / 90.).sin() * 2.).sin() * 0.5 + 0.5;
            put(if lum(q) / 255. > s { hi } else { lo })
        }),
        "oil" => each(px, &mut |q, _, _| {
            let k = |v: u8| ((v as f32 / 42.).round() * 42.).min(255.);
            put(Rgb((k(q.red()) * 1.08).min(255.) as u8, k(q.green()) as u8, k(q.blue()) as u8))
        }),
        "warm" => {
            let b = blurred(px, 4, 6);
            let mut c = Cv { px: std::mem::replace(px, Pixmap::new(1, 1).unwrap()), w: w as f32, h: h as f32, alpha: 0.45, blend: BlendMode::Screen,
                tf: tiny_skia::Transform::identity(), clip: None };
            c.draw_image(&b, 0., 0., w as f32, h as f32, true);
            c.blend = BlendMode::SoftLight;
            c.alpha = 0.5;
            c.fill_all(Cv::solid(col(Rgb(0xf2, 0xc4, 0x6d), 1.)));
            *px = c.px;
        }
        "dither" | "pixel" => {
            let k = if kind == "dither" { 3 } else { 8 };
            let (sw, sh) = (w.div_ceil(k) as u32, h.div_ceil(k) as u32);
            let mut s = Cv::new(sw, sh);
            s.draw_image(px, 0., 0., sw as f32, sh as f32, true);
            const B: [f32; 16] = [0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.];
            let pal = p.all();
            for (i, q) in s.px.pixels_mut().iter_mut().enumerate() {
                let (x, y) = (i % sw as usize, i / sw as usize);
                *q = if kind == "dither" {
                    put(if lum(*q) / 16. > B[(y % 4) * 4 + x % 4] + 0.5 { hi } else { lo })
                } else {
                    let d = |c: &Rgb| (c.0 as i32 - q.red() as i32).pow(2) + (c.1 as i32 - q.green() as i32).pow(2) + (c.2 as i32 - q.blue() as i32).pow(2);
                    put(*pal.iter().min_by_key(|c| d(c)).unwrap())
                };
            }
            let mut c = Cv::new(w as u32, h as u32);
            c.draw_image(&s.px, 0., 0., w as f32, h as f32, false);
            *px = c.px;
        }
        "halftone" => {
            let cell = ((w as f32 / 260.).round() as usize).max(4);
            let (sw, sh) = (w.div_ceil(cell) as u32, h.div_ceil(cell) as u32);
            let mut s = Cv::new(sw, sh);
            s.draw_image(px, 0., 0., sw as f32, sh as f32, true);
            let mut c = Cv::new(w as u32, h as u32);
            c.fill_all(Cv::solid(col(lo, 1.)));
            let dot = Cv::solid(col(if dark { hi } else { p.accent2 }, 1.));
            for (i, q) in s.px.pixels().iter().enumerate() {
                let r = lum(*q) / 255. * cell as f32 * 0.62;
                if r > 0.4 { c.circle(((i % sw as usize) * cell) as f32 + cell as f32 / 2., ((i / sw as usize) * cell) as f32 + cell as f32 / 2., r, dot.clone()); }
            }
            *px = c.px;
        }
        _ => {}
    }
}

pub fn compose(o: &Shot) -> Pixmap {
    let mut c = Cv::new(o.w, o.h);
    let (w, h) = (o.w as f32, o.h as f32);
    let imgs = o.images;
    if imgs.is_empty() {
        // nothing of the user's yet: the theme's own background
        backgrounds::draw(&mut c, o.theme);
        return c.px;
    }
    match o.layout {
        "triptych" => for k in 0..3 { cover(&mut c, &imgs[k % imgs.len()], k as f32 * w / 3., 0., w / 3. + 1., h) },
        "blur" => {
            let mut back = Cv::new(o.w, o.h);
            cover(&mut back, &imgs[0], -w * 0.05, -h * 0.05, w * 1.1, h * 1.1);
            let b = blurred(&back.px, 8, ((w / 60.) as u32 / 8).max(2));
            c.draw_image(&b, 0., 0., w, h, true);
            c.fill_all(Cv::solid(tiny_skia::Color::from_rgba8(0, 0, 0, 64)));
            contain(&mut c, &imgs[0], w / 2., h / 2., w, h);
        }
        "poster" => {
            backgrounds::draw(&mut c, o.theme);
            let (x, y, iw, ih) = {
                let s = (w * 0.8 / imgs[0].width() as f32).min(h * 0.8 / imgs[0].height() as f32);
                let (iw, ih) = (imgs[0].width() as f32 * s, imgs[0].height() as f32 * s);
                (w / 2. - iw / 2., h * 0.53 - ih / 2., iw, ih)
            };
            c.alpha = 0.45;
            c.fill_rect(x + w / 300., y + w / 200., iw, ih, Cv::solid(tiny_skia::Color::BLACK));
            c.alpha = 1.;
            contain(&mut c, &imgs[0], w / 2., h * 0.53, w * 0.8, h * 0.8);
            if let Some(r) = Rect::from_xywh(x - w / 200., y - w / 200., iw + w / 100., ih + w / 100.) {
                c.stroke(&PathBuilder::from_rect(r), Cv::solid(col(o.theme.palette.accent, 1.)), (w / 900.).max(2.));
            }
        }
        _ => cover(&mut c, &imgs[0], 0., 0., w, h),
    }
    if o.treatment != "none" && o.strength > 0. {
        let mut t = c.px.clone();
        treat(&mut t, o.treatment, o.theme);
        c.alpha = o.strength.min(1.);
        c.draw_image(&t, 0., 0., w, h, false);
    }
    c.px
}
