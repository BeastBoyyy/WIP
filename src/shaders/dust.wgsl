@group(0) @binding(0) var<uniform> u: Slab;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;
@group(1) @binding(0) var<uniform> f: Frame;
struct O { @builtin(position) pos: vec4<f32>, @location(0) c: vec4<f32>, @location(1) q: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> O {
    let s = f32(ii) * 1.618 + u.p.w * 97.0;
    let r1 = hash1(s);
    let r2 = hash1(s + 3.1);
    let r3 = hash1(s + 7.7);
    let r4 = hash1(s + 11.3);
    let life = fract(u.cam.w * (0.05 + r4 * 0.08) + r1);
    // born on the glass, biased toward its trailing edge, then blown off and curled
    let uv = vec2<f32>(pow(r2, 0.45), r3);
    let local = vec3<f32>((uv.x - 0.5) * u.size.x, (0.5 - uv.y) * u.size.y, 0.0);
    let n = normalize((u.model * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz);
    let right = normalize((u.model * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz);
    let born = (u.model * vec4<f32>(local, 1.0)).xyz;
    let k = life * life;
    let curl = vec3<f32>(sin(life * 6.0 + r1 * 20.0), cos(life * 5.0 + r3 * 17.0), sin(life * 4.0 + r2 * 13.0)) * 0.25 * life;
    let wp = born + right * k * (0.9 + r3 * 1.2) + n * k * 0.6 * (r1 - 0.3) + vec3<f32>(0.0, 0.35, 0.0) * k + curl;
    let clip = f.vp * vec4<f32>(wp, 1.0);
    let qd = quadv(vi);
    let px = (1.4 + r4 * 2.8) * (1.0 - life * 0.6) * (f.res.y / 900.0);
    var o: O;
    o.pos = vec4<f32>(clip.xy + qd * 4.0 * px / f.res.xy * clip.w, clip.zw);
    let col = textureSampleLevel(tex, smp, uv, 0.0).rgb;
    let fade = smoothstep(0.0, 0.08, life) * (1.0 - smoothstep(0.55, 1.0, life));
    o.c = vec4<f32>(col * 1.6 + u.rim.rgb * 0.25, fade * u.size.w * u.p.x);
    o.q = qd;
    return o;
}
@fragment fn fs(i: O) -> @location(0) vec4<f32> {
    let d = length(i.q) * 2.0;
    let a = i.c.a * (1.0 - smoothstep(0.2, 1.0, d));
    return vec4<f32>(i.c.rgb * a, 0.0);
}
