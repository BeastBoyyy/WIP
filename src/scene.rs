//! Motion for the stage: the cards ride a helix around a rotating tesseract, the camera spirals between them,
//! and springs carry it between pages (gallery, the exploded desk for Customize and Wallpapers, the core for Claude).
use crate::gpu::*;

pub const WALL: usize = 10;
pub const BAR: usize = 11;
pub const CLOCK: usize = 12;
const CW: f32 = 2.4;
const CH: f32 = 1.35;
const R: f32 = 4.0;
const STEP: f32 = 0.9;
const DROP: f32 = 0.8;
const FOV: f32 = 0.7;

#[derive(Clone, Copy)]
pub struct Spring { pub x: f32, pub v: f32, pub to: f32 }
impl Spring {
    pub fn new(x: f32) -> Spring { Spring { x, v: 0., to: x } }
    /// damped spring, `w` stiffness as angular frequency, `z` damping ratio (below 1 overshoots)
    fn step(&mut self, dt: f32, w: f32, z: f32) {
        let n = (dt / 0.004).ceil().max(1.);
        let h = dt / n;
        for _ in 0..n as usize {
            self.v += (w * w * (self.to - self.x) - 2. * z * w * self.v) * h;
            self.x += self.v * h;
        }
    }
    fn settled(&self) -> bool { (self.to - self.x).abs() < 1e-3 && self.v.abs() < 1e-3 }
    fn snap(&mut self) { self.x = self.to; self.v = 0.; }
}

fn smooth(t: f32) -> f32 { let t = t.clamp(0., 1.); t * t * (3. - 2. * t) }
fn out3(t: f32) -> f32 { 1. - (1. - t.clamp(0., 1.)).powi(3) }
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
fn lerp3(a: V3, b: V3, t: f32) -> V3 { [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)] }

/// Where the bar and clock sit on the wallpaper, as fractions: bar height, clock x, y, w, h.
pub type DeskGeo = [f32; 5];

pub struct Scene {
    pub n: usize,
    pub time: f32,
    pub focus: Spring,
    pub desk: Spring,
    pub core: Spring,
    tilt: [Spring; 2],
    intro: f32,
    pub hover: Option<usize>,
    hover_k: Vec<f32>,
    pointer: (f32, f32),
    veil: Option<(f32, f32, f32)>,
    sweep: Option<f32>,
    colours: [[f32; 3]; 4],
    pub target_colours: [[f32; 3]; 4],
    pub rims: Vec<[f32; 3]>,
    pub desk_geo: Option<DeskGeo>,
    pub reduced: bool,
    /// share of the window width the side panel covers
    pub panel: f32,
    pub last_input: f32,
    aspect: f32,
    eye: V3,
    vp: M4,
}

impl Scene {
    pub fn new(n: usize, reduced: bool) -> Scene {
        Scene { n, time: 0., focus: Spring::new(0.), desk: Spring::new(0.), core: Spring::new(0.), tilt: [Spring::new(0.); 2],
            intro: if reduced { 1. } else { 0. }, hover: None, hover_k: vec![0.; n], pointer: (0.5, 0.5), veil: None, sweep: None,
            colours: [[0.; 3]; 4], target_colours: [[0.; 3]; 4], rims: vec![[1.; 3]; n], desk_geo: None, reduced, panel: 0.3, last_input: 0.,
            aspect: 1.6, eye: [0.; 3], vp: [0.; 16] }
    }
    pub fn focused(&self) -> usize { self.focus.to.round() as usize }
    pub fn set_focus(&mut self, i: usize) { self.focus.to = i.min(self.n - 1) as f32; self.touch(); }
    /// 0 gallery, 1 customize, 2 wallpapers, 3 claude
    pub fn set_page(&mut self, p: i32) {
        self.desk.to = if p == 1 || p == 2 { 1. } else { 0. };
        self.core.to = if p == 3 { 1. } else { 0. };
        self.touch();
    }
    pub fn touch(&mut self) { self.last_input = self.time; }
    pub fn pointer(&mut self, x: f32, y: f32) {
        self.pointer = (x, y);
        self.tilt[0].to = x * 2. - 1.;
        self.tilt[1].to = y * 2. - 1.;
        self.hover = if self.desk.to < 0.5 && self.core.to < 0.5 { self.hit(x, y) } else { None };
        self.touch();
    }
    pub fn start_veil(&mut self, x: f32, y: f32) { if !self.reduced { self.veil = Some((x, y, self.time)); } self.touch(); }
    pub fn start_sweep(&mut self) { self.sweep = Some(self.time); self.touch(); }

    /// Something is still moving: render at full rate.
    pub fn busy(&self) -> bool {
        self.intro < 1. || self.veil.is_some() || self.sweep.is_some() || self.time - self.last_input < 3.
            || ![self.focus, self.desk, self.core, self.tilt[0], self.tilt[1]].iter().all(|s| s.settled())
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        self.time += if self.reduced { dt * 0.25 } else { dt };
        if self.reduced {
            for s in [&mut self.focus, &mut self.desk, &mut self.core].into_iter().chain(self.tilt.iter_mut()) { s.snap(); }
        } else {
            self.focus.step(dt, 7., 0.78);
            self.desk.step(dt, 6., 0.85);
            self.core.step(dt, 4.5, 0.9);
            for t in &mut self.tilt { t.step(dt, 5., 1.); }
            self.intro = (self.intro + dt / 3.).min(1.);
        }
        let k = 1. - (-dt * 5.).exp();
        for (c, t) in self.colours.iter_mut().zip(self.target_colours) { for j in 0..3 { c[j] += (t[j] - c[j]) * k; } }
        for (i, h) in self.hover_k.iter_mut().enumerate() { *h += ((self.hover == Some(i)) as u8 as f32 - *h) * (1. - (-dt * 10.).exp()); }
        if self.veil.is_some_and(|v| self.time - v.2 > 1.7) { self.veil = None; }
        if self.sweep.is_some_and(|s| self.time - s > 1.6) { self.sweep = None; }
    }

    fn camera(&self) -> (V3, V3) {
        let f = self.focus.x;
        let (th, y) = (f * STEP, -f * DROP);
        let e = out3(self.intro);
        let c = smooth(self.core.x);
        // narrow windows pull the camera back so the focused card stays in frame
        let fit = (1.6 / self.aspect).max(1.).powf(0.9);
        let dist = lerp(lerp(2.9, 10. * fit, e), 4.4 * fit, c);
        let ang = th - 0.45 - (1. - e) * 1.6 + c * 0.45;
        let eye = [ang.sin() * dist, y + 0.55 + (1. - e) * 0.9 - c * 0.35, ang.cos() * dist];
        (eye, [0., y - 0.1, 0.])
    }

    /// Model matrix, opacity, closeness to focus for card i.
    fn card(&self, i: usize, eye: V3, at: V3) -> (M4, f32, f32) {
        let fi = self.focused();
        let df = i as f32 - self.focus.x;
        let close = (1. - df.abs()).max(0.);
        let d = smooth(self.desk.x);
        let c = smooth(self.core.x);
        let th = i as f32 * STEP;
        // Claude page: the ring opens wide and dims so the tesseract has the stage
        let r = R + if i == fi { 0. } else { d * 3. } + c * 7.;
        let helix = [th.sin() * r, -(i as f32) * DROP, th.cos() * r];
        // the intro deals the cards out from the core, nearest first
        let deal = out3((self.intro * 1.6 - df.abs() * 0.08).clamp(0., 1.));
        let pos = lerp3([0., at[1], 0.], helix, deal);
        let mut yaw = th - 0.45 * close * 0.6 + (1. - deal) * 2.;
        let mut k = 1. + 0.18 * close;
        let mut op = (1.2 - df.abs() * 0.2).clamp(0.12, 1.) * deal * (1. - c * 0.75);
        let mut model = mul(&translate(pos), &mul(&rot_y(yaw), &scale(k)));
        if i == fi && d > 0. {
            // fly to the front, facing the camera, left of the side panel; tilt with the pointer
            let fw = norm(sub(at, eye));
            let rt = norm(cross(fw, [0., 1., 0.]));
            let up = cross(rt, fw);
            // sized to fill most of the space left of the panel, centred in it
            let (tn, ff) = ((FOV / 2.).tan(), 1. - self.panel);
            let dist = CW * 1.6 / (0.8 * ff * 2. * tn * self.aspect);
            let off = (ff / 2. - 0.5) * 2. * dist * tn * self.aspect;
            let desk_pos = add(add(eye, scl(fw, dist)), add(scl(rt, off), scl(up, -0.02 * dist)));
            let mut face = (-fw[0]).atan2(-fw[2]);
            face += std::f32::consts::TAU * ((yaw - face) / std::f32::consts::TAU).round();
            let p = lerp3(pos, desk_pos, d);
            // a resting three-quarter turn so the layers read as depth, not misregistration
            yaw = lerp(yaw, face + 0.3 + self.tilt[0].x * 0.3, d);
            let pitch = (-0.14 + self.tilt[1].x * 0.22) * d;
            k = lerp(k, 1.6, d);
            op = lerp(op, 1., d);
            model = mul(&translate(p), &mul(&rot_y(yaw), &mul(&rot_x(pitch), &scale(k))));
        }
        if i != fi { op *= 1. - d; }
        (model, op, close)
    }

    fn slab(&self, model: M4, w: f32, h: f32, radius: f32, op: f32, glow: f32, sweep: f32, seed: f32, dust: f32, rim: [f32; 3], hover: f32) -> SlabU {
        SlabU { mvp: mul(&self.vp, &model), model, cam: [self.eye[0], self.eye[1], self.eye[2], self.time], p: [op, glow, sweep, seed],
            size: [w, h, radius, dust], rim: [rim[0], rim[1], rim[2], hover] }
    }

    /// Ray from a pointer position (0..1) to the nearest card under it.
    pub fn hit(&self, x: f32, y: f32) -> Option<usize> {
        let (eye, at) = self.camera();
        let fw = norm(sub(at, eye));
        let rt = norm(cross(fw, [0., 1., 0.]));
        let up = cross(rt, fw);
        let t = (FOV / 2.).tan();
        let dir = norm(add(fw, add(scl(rt, (x * 2. - 1.) * t * self.aspect), scl(up, (1. - y * 2.) * t))));
        let mut best: Option<(f32, usize)> = None;
        for i in 0..self.n {
            let (m, op, _) = self.card(i, eye, at);
            if op < 0.2 { continue; }
            let col = |c: usize| -> V3 { [m[c * 4], m[c * 4 + 1], m[c * 4 + 2]] };
            let (ax, ay, n, o) = (col(0), col(1), col(2), col(3));
            let denom = dot(dir, n);
            if denom.abs() < 1e-5 { continue; }
            let d = dot(sub(o, eye), n) / denom;
            if d <= 0. { continue; }
            let rel = sub(add(eye, scl(dir, d)), o);
            let (lx, ly) = (dot(rel, ax) / dot(ax, ax), dot(rel, ay) / dot(ay, ay));
            if lx.abs() <= CW / 2. && ly.abs() <= CH / 2. && best.is_none_or(|b| d < b.0) { best = Some((d, i)); }
        }
        best.map(|b| b.1)
    }

    // ponytail: placeholder centrepiece; the 3D Rice Studio logo replaces it once designed (same spot, same glow and core zoom)
    fn tesseract(&self, at: V3) -> Vec<[f32; 8]> {
        let t = self.time;
        let c = smooth(self.core.x);
        let spin = t * (1. + c * 1.5);
        let (a, b, g) = (spin * 0.35, spin * 0.21, spin * 0.13);
        let size = lerp(1.15, 1.4, c) * (0.6 + 0.4 * out3(self.intro));
        let ry = t * 0.1;
        let v: Vec<([f32; 3], f32)> = (0..16).map(|k| {
            let p = [0, 1, 2, 3].map(|j| if k >> j & 1 == 1 { 1f32 } else { -1. });
            let (mut x, mut y, mut z, mut w) = (p[0], p[1], p[2], p[3]);
            (x, w) = (x * a.cos() - w * a.sin(), x * a.sin() + w * a.cos());
            (y, z) = (y * b.cos() - z * b.sin(), y * b.sin() + z * b.cos());
            (z, w) = (z * g.cos() - w * g.sin(), z * g.sin() + w * g.cos());
            let k4 = 1. / (3.5 - w);
            let (x, z) = (x * ry.cos() + z * ry.sin(), -x * ry.sin() + z * ry.cos());
            ([at[0] + x * k4 * size * 2., at[1] + 0.05 + (t * 0.6).sin() * 0.06 + y * k4 * size * 2., at[2] + z * k4 * size * 2.], w / 2.)
        }).collect();
        let mut e = Vec::with_capacity(32);
        for i in 0..16usize { for j in 0..4 { let k = i ^ (1 << j); if i < k {
            let (p, q) = (v[i], v[k]);
            e.push([p.0[0], p.0[1], p.0[2], p.1, q.0[0], q.0[1], q.0[2], q.1]);
        } } }
        e
    }

    pub fn draw(&mut self, w: u32, h: u32, has_desk: bool) -> Draw {
        self.aspect = w as f32 / h.max(1) as f32;
        let (eye, at) = self.camera();
        self.eye = eye;
        self.vp = mul(&perspective(FOV, self.aspect, 0.1, 80.), &look_at(eye, at));
        let fi = self.focused();
        let fw = norm(sub(at, eye));
        let sweep = self.sweep.map(|s| ((self.time - s) / 1.4).min(1.)).unwrap_or(-1.);
        let dust_k = if self.reduced { 0. } else { 1. };
        let mut slabs: Vec<(f32, usize, SlabU, u32)> = Vec::new();
        for i in 0..self.n {
            let (m, op, close) = self.card(i, eye, at);
            if op < 0.01 { continue; }
            let depth = dot(sub([m[12], m[13], m[14]], eye), fw);
            let rim = self.rims[i];
            let hov = self.hover_k[i];
            let seed = i as f32 * 0.137;
            let sw = if i == fi { sweep } else { -1. };
            let n_dust = if close > 0.5 { 240. } else if close > 0. { 90. } else { 0. } * dust_k * (1. - self.core.x * 0.5);
            let d = self.desk.x;
            match self.desk_geo {
                Some(g) if i == fi && has_desk && d > 0.02 => {
                    // the card comes apart into wallpaper, bar and clock
                    slabs.push((depth, WALL, self.slab(m, CW, CH, 0.04, op, close, sw, seed, 0.5, rim, hov), (n_dust * 0.5) as u32));
                    let bar = mul(&m, &translate([0., CH / 2. - CH * g[0] / 2., d * 0.28]));
                    slabs.push((depth - 0.3 * d, BAR, self.slab(bar, CW, CH * g[0], 0.001, op, 0., sw, seed, 0., [0.; 3], 0.), 0));
                    let clk = mul(&m, &translate([CW * (g[1] + g[3] / 2. - 0.5), CH * (0.5 - g[2] - g[4] / 2.), d * 0.5]));
                    slabs.push((depth - 0.6 * d, CLOCK, self.slab(clk, CW * g[3], CH * g[4], 0.001, op, 0., sw, seed, 0., [0.; 3], 0.), 0));
                }
                _ => slabs.push((depth, i, self.slab(m, CW, CH, 0.06, op, close, sw, seed, 1., rim, hov), n_dust as u32)),
            }
        }
        slabs.sort_by(|a, b| b.0.total_cmp(&a.0));
        let veil = self.veil.map(|(x, y, t0)| {
            let el = self.time - t0;
            [x * w as f32, y * h as f32, out3(el / 0.6), ((el - 0.75) / 0.9).clamp(0., 1.)]
        });
        let [bg, a1, a2, ink] = self.colours;
        let glow = 1. + smooth(self.core.x) * 0.8 + (1. - out3(self.intro)) * 0.8;
        Draw {
            frame: FrameU { vp: self.vp, res: [w as f32, h as f32, self.time, self.focus.x],
                bg: [bg[0], bg[1], bg[2], 1.], a1: [a1[0], a1[1], a1[2], 1.], a2: [a2[0], a2[1], a2[2], 1.], ink: [ink[0], ink[1], ink[2], glow],
                veil: veil.unwrap_or_default() },
            slabs: slabs.into_iter().map(|s| (s.1, s.2, s.3)).collect(),
            lines: self.tesseract([0., at[1], 0.]),
            veil: veil.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_focused_card_is_hit_where_it_is_drawn() {
        let mut s = Scene::new(10, true);
        s.set_focus(3);
        s.tick(0.016);
        let d = s.draw(1600, 1000, false);
        // project the focused card's centre and click there
        let (_, u, _) = d.slabs.iter().find(|x| x.0 == 3).unwrap();
        let c: Vec<f32> = (0..4).map(|r| u.mvp[12 + r]).collect();
        let (x, y) = ((c[0] / c[3] + 1.) / 2., (1. - c[1] / c[3]) / 2.);
        assert!(x > 0.5, "focused card sits right of centre, got {x}");
        assert_eq!(s.hit(x, y), Some(3));
        assert_eq!(d.lines.len(), 32);
    }
}
