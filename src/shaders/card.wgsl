@group(0) @binding(0) var<uniform> u: Slab;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;
struct O { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) world: vec3<f32>, @location(2) n: vec3<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> O {
    let q = quadv(vi);
    let local = vec4<f32>(q.x * u.size.x, q.y * u.size.y, 0.0, 1.0);
    var o: O;
    o.pos = u.mvp * local;
    o.uv = vec2<f32>(q.x + 0.5, 0.5 - q.y);
    o.world = (u.model * local).xyz;
    o.n = normalize((u.model * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz);
    return o;
}
@fragment fn fs(i: O) -> @location(0) vec4<f32> {
    let v = normalize(u.cam.xyz - i.world);
    let facing = dot(i.n, v);
    let hs = u.size.xy * 0.5;
    let lp = (i.uv - 0.5) * u.size.xy;
    let d = length(max(abs(lp) - hs + u.size.z, vec2<f32>(0.0))) - u.size.z;
    let aa = max(fwidth(d), 1e-4);
    let mask = 1.0 - smoothstep(-aa, aa, d);
    var uv = i.uv;
    if (facing < 0.0) { uv.x = 1.0 - uv.x; }
    // chromatic split grows toward the rim
    let edge = smoothstep(-0.18, 0.0, d);
    let ca = (uv - 0.5) * 0.012 * edge;
    let mid = textureSample(tex, smp, uv);
    var c = vec3<f32>(textureSample(tex, smp, uv + ca).r, mid.g, textureSample(tex, smp, uv - ca).b);
    let ta = mid.a;
    let fres = pow(1.0 - abs(facing), 2.5);
    if (facing < 0.0) { c = c * 0.18; }
    // a slow sheen band across the glass
    let band = uv.x * 0.8 + uv.y * 0.5 - (fract(u.cam.w * 0.07 + u.p.w) * 2.6 - 0.8);
    c += vec3<f32>(smoothstep(0.09, 0.0, abs(band)) * 0.10 * ta);
    // rim light, stronger on the chosen and the hovered card
    let rimw = smoothstep(aa * 4.0, 0.0, abs(d));
    c += u.rim.rgb * (rimw * (0.35 + u.p.y * 1.2 + u.rim.a * 0.8) + fres * 0.5 * (0.4 + u.p.y));
    // apply sweep: a bright line develops the picture top to bottom
    if (u.p.z >= 0.0) {
        let line = smoothstep(0.012, 0.0, abs(uv.y - u.p.z));
        c = mix(c * 0.25, c, step(uv.y, u.p.z)) + u.rim.rgb * line * 2.0;
    }
    let a = mask * u.p.x * max(ta, rimw * step(0.001, dot(u.rim.rgb, vec3<f32>(1.0))));
    if (a < 0.004) { discard; }
    return vec4<f32>(c * mask * u.p.x, a);
}
