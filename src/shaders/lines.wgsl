@group(0) @binding(0) var<uniform> f: Frame;
struct I { @location(0) p0: vec4<f32>, @location(1) p1: vec4<f32> };   // xyz + the 4th coordinate
struct O { @builtin(position) pos: vec4<f32>, @location(0) side: f32, @location(1) w: f32, @location(2) along: f32 };
@vertex fn vs(@builtin(vertex_index) vi: u32, e: I) -> O {
    let q = quadv(vi);
    let c0 = f.vp * vec4<f32>(e.p0.xyz, 1.0);
    let c1 = f.vp * vec4<f32>(e.p1.xyz, 1.0);
    let s0 = c0.xy / c0.w * f.res.xy * 0.5;
    let s1 = c1.xy / c1.w * f.res.xy * 0.5;
    let dir = normalize(s1 - s0 + vec2<f32>(1e-5, 0.0));
    let nrm = vec2<f32>(-dir.y, dir.x);
    let t = q.x + 0.5;
    let c = mix(c0, c1, t);
    let width = 9.0 * f.ink.a * (f.res.y / 900.0);
    let off = (nrm * q.y * width + dir * (t - 0.5) * width * 0.6) / (f.res.xy * 0.5) * c.w;
    var o: O;
    o.pos = vec4<f32>(c.xy + off, c.z, c.w);
    o.side = q.y * 2.0;
    o.w = mix(e.p0.w, e.p1.w, t);
    o.along = t;
    return o;
}
@fragment fn fs(i: O) -> @location(0) vec4<f32> {
    let core = exp(-i.side * i.side * 18.0);
    let halo = exp(-i.side * i.side * 3.0) * 0.35;
    // the 4th dimension reads as colour: one accent near, the other far
    let k = clamp(i.w * 0.5 + 0.5, 0.0, 1.0);
    let col = mix(f.a1.rgb, f.a2.rgb, k);
    let shimmer = 0.75 + 0.25 * sin(i.along * 12.0 - f.res.z * 3.0 + i.w * 4.0);
    let rgb = col * (core * 1.4 + halo) * shimmer + f.ink.rgb * core * 0.35;
    return vec4<f32>(rgb, 0.0);
}
