@group(0) @binding(0) var<uniform> u: Frame;
struct O { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> O {
    let p = tri(vi);
    var o: O;
    o.pos = vec4<f32>(p, 1.0, 1.0);
    o.uv = p * 0.5 + 0.5;
    return o;
}
@fragment fn fs(i: O) -> @location(0) vec4<f32> {
    let t = u.res.z;
    let asp = u.res.x / u.res.y;
    let p = vec2<f32>(i.uv.x * asp, i.uv.y + u.res.w * 0.08);
    // domain-warped nebula in the theme's two accents
    let q = vec2<f32>(fbm(p * 1.6 + vec2<f32>(0.0, t * 0.03)), fbm(p * 1.6 + vec2<f32>(5.2, 1.3) - t * 0.02));
    let n = fbm(p * 2.2 + q * 2.4 + vec2<f32>(t * 0.015, 0.0));
    let n2 = fbm(p * 3.1 - q * 1.7 + 4.0);
    let v = smoothstep(0.0, 1.0, length(i.uv - vec2<f32>(0.55, 0.5)) * 1.3);
    var c = mix(u.bg.rgb * 0.35, u.bg.rgb * 0.08, v);
    c += u.a1.rgb * pow(n, 3.2) * 0.55 + u.a2.rgb * pow(n2, 4.0) * 0.45;
    // far stars, parallaxed by the helix scroll
    let sp = (i.uv + vec2<f32>(0.0, u.res.w * 0.03)) * vec2<f32>(asp, 1.0) * 140.0;
    let st = hash2(floor(sp));
    let tw = 0.5 + 0.5 * sin(t * (1.0 + st * 3.0) + st * 40.0);
    c += u.ink.rgb * step(0.996, st) * tw * (1.0 - smoothstep(0.0, 0.45, length(fract(sp) - 0.5))) * 0.8;
    // grain
    c += (hash2(i.uv * u.res.xy + fract(t) * 91.0) - 0.5) * 0.035;
    return vec4<f32>(c, 1.0);
}
