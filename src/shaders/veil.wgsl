@group(0) @binding(0) var<uniform> u: Frame;
struct O { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> O {
    let p = tri(vi);
    var o: O;
    o.pos = vec4<f32>(p, 0.0, 1.0);
    o.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return o;
}
@fragment fn fs(i: O) -> @location(0) vec4<f32> {
    let px = i.uv * u.res.xy;
    let d = length(px - u.veil.xy) / length(u.res.xy);
    let n = fbm(i.uv * vec2<f32>(u.res.x / u.res.y, 1.0) * 5.0 + u.res.z * 0.4);
    // ink spreads from the click with a ragged, wet edge
    let r = u.veil.z * 1.25 - 0.1;
    let dn = d + (n - 0.5) * 0.22;
    let inside = smoothstep(r + 0.03, r - 0.01, dn);
    // then dries away in blotches
    let dry = smoothstep(u.veil.w - 0.06, u.veil.w + 0.06, n * 0.85 + d * 0.3);
    let a = inside * mix(1.0, dry, step(0.001, u.veil.w));
    let swirl = fbm(i.uv * 3.0 + n * 2.0 - u.res.z * 0.2);
    var c = mix(u.bg.rgb, u.a1.rgb, smoothstep(0.35, 0.9, swirl) * 0.8);
    c = mix(c, u.a2.rgb, smoothstep(0.6, 1.0, n) * 0.5);
    c += u.a1.rgb * smoothstep(0.05, 0.0, abs(dn - r)) * (1.0 - u.veil.w) * 1.5;
    return vec4<f32>(c * a, a);
}
