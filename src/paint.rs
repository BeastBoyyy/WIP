//! A small 2D canvas over tiny-skia: the CPU painter for procedural backgrounds, wallpapers, bar previews
//! and card faces. Text is drawn from real glyph outlines, so any bundled family renders at any size.
use crate::theme::Rgb;
use tiny_skia::*;

pub struct Cv {
    pub px: Pixmap,
    pub w: f32,
    pub h: f32,
    pub alpha: f32,
    pub blend: BlendMode,
    pub tf: Transform,
    pub clip: Option<Mask>,
}

pub fn col(c: Rgb, a: f32) -> Color { Color::from_rgba8(c.0, c.1, c.2, (a.clamp(0., 1.) * 255.) as u8) }
pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color { Color::from_rgba8(r, g, b, (a.clamp(0., 1.) * 255.) as u8) }

/// Park-Miller, same sequence as the TS prototype so the pictures match.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed.max(1)) }
    pub fn f(&mut self) -> f32 { self.0 = self.0 * 16807 % 2147483647; self.0 as f32 / 2147483647.0 }
}

impl Cv {
    pub fn new(w: u32, h: u32) -> Cv {
        Cv { px: Pixmap::new(w.max(1), h.max(1)).unwrap(), w: w as f32, h: h as f32, alpha: 1., blend: BlendMode::SourceOver, tf: Transform::identity(), clip: None }
    }
    fn paint<'a>(&self, shader: Shader<'a>) -> Paint<'a> {
        let mut p = Paint { shader, anti_alias: true, blend_mode: self.blend, ..Default::default() };
        if self.alpha < 1. { p.shader.apply_opacity(self.alpha); }
        p
    }
    pub fn solid(c: Color) -> Shader<'static> { Shader::SolidColor(c) }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, s: Shader) {
        if let Some(r) = Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
            let p = self.paint(s);
            self.px.fill_rect(r, &p, self.tf, self.clip.as_ref());
        }
    }
    pub fn fill_all(&mut self, s: Shader) { let (w, h) = (self.w, self.h); let t = self.tf; self.tf = Transform::identity(); self.fill_rect(0., 0., w, h, s); self.tf = t; }
    pub fn fill(&mut self, path: &Path, s: Shader) {
        let p = self.paint(s);
        self.px.fill_path(path, &p, FillRule::Winding, self.tf, self.clip.as_ref());
    }
    pub fn stroke(&mut self, path: &Path, s: Shader, width: f32) {
        let p = self.paint(s);
        let st = Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Default::default() };
        self.px.stroke_path(path, &p, &st, self.tf, self.clip.as_ref());
    }
    pub fn circle(&mut self, x: f32, y: f32, r: f32, s: Shader) {
        if r > 0.05 { if let Some(p) = PathBuilder::from_circle(x, y, r) { self.fill(&p, s); } }
    }
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, s: Shader, w: f32) {
        let mut b = PathBuilder::new(); b.move_to(x0, y0); b.line_to(x1, y1);
        if let Some(p) = b.finish() { self.stroke(&p, s, w); }
    }
    pub fn clip_path(&mut self, path: &Path) {
        let mut m = Mask::new(self.px.width(), self.px.height()).unwrap();
        m.fill_path(path, FillRule::Winding, true, self.tf);
        self.clip = Some(m);
    }

    pub fn linear(x0: f32, y0: f32, x1: f32, y1: f32, stops: &[(f32, Color)]) -> Shader<'static> {
        LinearGradient::new(Point::from_xy(x0, y0), Point::from_xy(x1, y1), stops.iter().map(|s| GradientStop::new(s.0, s.1)).collect(),
            SpreadMode::Pad, Transform::identity()).unwrap_or(Shader::SolidColor(stops[0].1))
    }
    pub fn radial(cx: f32, cy: f32, r: f32, stops: &[(f32, Color)]) -> Shader<'static> {
        RadialGradient::new(Point::from_xy(cx, cy), 0., Point::from_xy(cx, cy), r.max(0.1), stops.iter().map(|s| GradientStop::new(s.0, s.1)).collect(),
            SpreadMode::Pad, Transform::identity()).unwrap_or(Shader::SolidColor(stops[0].1))
    }
    /// soft round blob, colour fading to transparent
    pub fn cloud(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32) {
        let s = Self::radial(x, y, r, &[(0., col(c, a)), (1., col(c, 0.))]);
        self.circle(x, y, r, s);
    }
    pub fn vignette(&mut self, c: Rgb, a: f32) {
        let (w, h) = (self.w, self.h);
        let r1 = w.min(h) * 0.3 / (w.max(h) * 0.75);
        let s = Self::radial(w / 2., h / 2., w.max(h) * 0.75, &[(0., col(c, 0.)), (r1, col(c, 0.)), (1., col(c, a))]);
        self.fill_all(s);
    }
    /// film/toner noise: random-alpha speckle of one colour
    pub fn noise(&mut self, a: f32, c: Rgb, seed: u32) {
        let mut s = seed.wrapping_mul(2654435761).max(1);
        let a = a.clamp(0., 1.);
        for p in self.px.pixels_mut() {
            s ^= s << 13; s ^= s >> 17; s ^= s << 5;
            let k = a * ((s & 255) as f32 / 255.);
            let (r, g, b, al) = (p.red() as f32, p.green() as f32, p.blue() as f32, p.alpha());
            let m = |x: f32, y: u8| (x + (y as f32 * al as f32 / 255. - x) * k) as u8;
            *p = PremultipliedColorU8::from_rgba(m(r, c.0), m(g, c.1), m(b, c.2), al).unwrap_or(*p);
        }
    }
    pub fn arc(cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) -> Option<Path> {
        let mut b = PathBuilder::new();
        let n = (((a1 - a0).abs() * rx.max(ry)) / 4.).clamp(12., 400.) as usize;
        for i in 0..=n {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            let (x, y) = (cx + a.cos() * rx, cy + a.sin() * ry);
            if i == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
        }
        b.finish()
    }
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32) -> Option<Path> {
        let p = PathBuilder::from_oval(Rect::from_xywh(-rx, -ry, rx * 2., ry * 2.)?)?;
        p.transform(Transform::from_rotate(rot.to_degrees()).post_translate(cx, cy))
    }

    /// Draw text with its baseline at y. Returns the advance width.
    pub fn text(&mut self, s: &str, family: &str, size: f32, x: f32, y: f32, c: Color) -> f32 {
        let (path, w) = text_path(s, family, size, x, y);
        if let Some(p) = path { self.fill(&p, Shader::SolidColor(c)); }
        w
    }
    /// Fit text to a width (headline posters); y is the top. Returns the size used.
    pub fn headline(&mut self, s: &str, family: &str, c: Color, x: f32, y: f32, max_w: f32) -> f32 {
        let w0 = measure(s, family, 100.);
        let size = if w0 > 0. { 100. * max_w / w0 } else { max_w / 3. };
        self.text(s, family, size, x, y + size * 0.72, c);
        size
    }
    pub fn draw_image(&mut self, img: &Pixmap, x: f32, y: f32, w: f32, h: f32, smooth: bool) {
        let sx = w / img.width() as f32;
        let sy = h / img.height() as f32;
        let mut paint = PixmapPaint { quality: if smooth { FilterQuality::Bilinear } else { FilterQuality::Nearest }, blend_mode: self.blend, ..Default::default() };
        paint.opacity = self.alpha;
        let t = self.tf.pre_translate(x, y).pre_scale(sx, sy);
        self.px.draw_pixmap(0, 0, img.as_ref(), &paint, t, self.clip.as_ref());
    }
    pub fn reset(&mut self) { self.alpha = 1.; self.blend = BlendMode::SourceOver; self.tf = Transform::identity(); self.clip = None; }
}

struct Outline { b: PathBuilder, s: f32, x: f32, y: f32 }
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) { self.b.move_to(self.x + x * self.s, self.y - y * self.s) }
    fn line_to(&mut self, x: f32, y: f32) { self.b.line_to(self.x + x * self.s, self.y - y * self.s) }
    fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) { self.b.quad_to(self.x + a * self.s, self.y - b * self.s, self.x + x * self.s, self.y - y * self.s) }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.b.cubic_to(self.x + a * self.s, self.y - b * self.s, self.x + c * self.s, self.y - d * self.s, self.x + x * self.s, self.y - y * self.s)
    }
    fn close(&mut self) { self.b.close() }
}

fn faces(family: &str) -> (Option<ttf_parser::Face<'static>>, Option<ttf_parser::Face<'static>>) {
    let f = crate::fonts::get();
    let main = f.by_family.get(family).or_else(|| f.by_family.get("Inter")).and_then(|b| ttf_parser::Face::parse(b, 0).ok());
    let cjk = f.cjk.and_then(|(b, i)| ttf_parser::Face::parse(b, i).ok());
    (main, cjk)
}

pub fn text_path(s: &str, family: &str, size: f32, x: f32, y: f32) -> (Option<Path>, f32) {
    let (main, cjk) = faces(family);
    let mut o = Outline { b: PathBuilder::new(), s: 0., x, y };
    for ch in s.chars() {
        let face = match (&main, &cjk) {
            (Some(m), _) if m.glyph_index(ch).is_some() => m,
            (_, Some(c)) if c.glyph_index(ch).is_some() => c,
            (Some(m), _) => m,
            _ => return (None, 0.),
        };
        let Some(g) = face.glyph_index(ch) else { o.x += size * 0.3; continue };
        o.s = size / face.units_per_em() as f32;
        face.outline_glyph(g, &mut o);
        o.x += face.glyph_hor_advance(g).unwrap_or(0) as f32 * o.s;
    }
    (o.b.finish(), o.x - x)
}
pub fn measure(s: &str, family: &str, size: f32) -> f32 { text_path(s, family, size, 0., 0.).1 }

/// Cheap blur: shrink, box-blur three times, scale back up.
pub fn blurred(src: &Pixmap, shrink: u32, radius: u32) -> Pixmap {
    let (w, h) = ((src.width() / shrink).max(1), (src.height() / shrink).max(1));
    let mut small = Cv::new(w, h);
    small.draw_image(src, 0., 0., w as f32, h as f32, true);
    let mut d = small.px.data().to_vec();
    for _ in 0..3 { box_pass(&mut d, w as usize, h as usize, radius as usize, true); box_pass(&mut d, w as usize, h as usize, radius as usize, false); }
    small.px.data_mut().copy_from_slice(&d);
    let mut out = Cv::new(src.width(), src.height());
    out.draw_image(&small.px, 0., 0., src.width() as f32, src.height() as f32, true);
    out.px
}
fn box_pass(d: &mut [u8], w: usize, h: usize, r: usize, horiz: bool) {
    let (len, lines) = if horiz { (w, h) } else { (h, w) };
    let idx = |line: usize, i: usize| if horiz { (line * w + i) * 4 } else { (i * w + line) * 4 };
    let mut buf = vec![0u8; len * 4];
    for line in 0..lines {
        for c in 0..4 {
            let mut acc = 0u32;
            let n = (2 * r + 1) as u32;
            for k in 0..=2 * r { acc += d[idx(line, (k as isize - r as isize).clamp(0, len as isize - 1) as usize) + c] as u32; }
            for i in 0..len {
                buf[i * 4 + c] = (acc / n) as u8;
                let add = (i + r + 1).min(len - 1);
                let sub = (i as isize - r as isize).max(0) as usize;
                acc = acc + d[idx(line, add) + c] as u32 - d[idx(line, sub) + c] as u32;
            }
        }
        for i in 0..len { d[idx(line, i)..idx(line, i) + 4].copy_from_slice(&buf[i * 4..i * 4 + 4]); }
    }
}

pub fn to_pixmap(img: &image::RgbaImage) -> Pixmap {
    let mut p = Pixmap::new(img.width().max(1), img.height().max(1)).unwrap();
    for (dst, src) in p.pixels_mut().iter_mut().zip(img.pixels()) {
        *dst = ColorU8::from_rgba(src[0], src[1], src[2], src[3]).premultiply();
    }
    p
}
pub fn to_png(p: &Pixmap) -> Vec<u8> { p.encode_png().unwrap_or_default() }
