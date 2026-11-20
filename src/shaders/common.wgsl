// Shared by every stage shader (prepended at load).
struct Frame {
    vp: mat4x4<f32>,
    res: vec4<f32>,    // target w, h, time, scroll
    bg: vec4<f32>,
    a1: vec4<f32>,
    a2: vec4<f32>,
    ink: vec4<f32>,    // rgb, line glow
    veil: vec4<f32>,   // x, y (px), grow 0..1, dry 0..1
};
struct Slab {
    mvp: mat4x4<f32>,
    model: mat4x4<f32>,
    cam: vec4<f32>,    // camera position, time
    p: vec4<f32>,      // opacity, glow, sweep (<0 off), seed
    size: vec4<f32>,   // w, h, corner radius, dust rate
    rim: vec4<f32>,    // rim colour, hover
};

fn pcg(v: u32) -> u32 { let s = v * 747796405u + 2891336453u; let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u; return (w >> 22u) ^ w; }
// integer hashes: sin()-based ones lose precision on GPUs and turn the fog blocky
fn hash2(p: vec2<f32>) -> f32 { let i = vec2<i32>(floor(p)); return f32(pcg(bitcast<u32>(i.x) + pcg(bitcast<u32>(i.y))) >> 8u) / 16777215.0; }
fn hash1(n: f32) -> f32 { return f32(pcg(bitcast<u32>(n)) >> 8u) / 16777215.0; }
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}
fn fbm(p0: vec2<f32>) -> f32 {
    var p = p0;
    var a = 0.5;
    var s = 0.0;
    for (var i = 0; i < 5; i++) {
        s += a * vnoise(p);
        p = p * 2.03 + vec2<f32>(1.7, 9.2);
        a *= 0.5;
    }
    return s;
}
// fullscreen triangle
fn tri(vi: u32) -> vec2<f32> {
    return vec2<f32>(f32((vi << 1u) & 2u) * 2.0 - 1.0, f32(vi & 2u) * 2.0 - 1.0);
}
// unit quad, two triangles, centred on 0
fn quadv(vi: u32) -> vec2<f32> {
    var c = array<vec2<f32>, 6>(vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5),
                                vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, 0.5), vec2<f32>(-0.5, 0.5));
    return c[vi];
}
