//! The 3D stage: a wgpu renderer drawing into a texture that Slint shows behind the UI.
//! Fog backdrop, glass card slabs, pigment dust, the tesseract lines, and the ink veil overlay.
use slint::wgpu_30::wgpu;
use wgpu::*;

pub type V3 = [f32; 3];
/// column-major 4x4
pub type M4 = [f32; 16];

pub fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
pub fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
pub fn scl(a: V3, k: f32) -> V3 { [a[0] * k, a[1] * k, a[2] * k] }
pub fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
pub fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
pub fn norm(a: V3) -> V3 { scl(a, 1. / dot(a, a).sqrt().max(1e-6)) }

pub fn mul(a: &M4, b: &M4) -> M4 {
    let mut r = [0.; 16];
    for c in 0..4 { for row in 0..4 { r[c * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[c * 4 + k]).sum(); } }
    r
}
pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> M4 {
    let f = 1. / (fovy / 2.).tan();
    [f / aspect, 0., 0., 0., 0., f, 0., 0., 0., 0., far / (near - far), -1., 0., 0., near * far / (near - far), 0.]
}
pub fn look_at(eye: V3, at: V3) -> M4 {
    let f = norm(sub(at, eye));
    let s = norm(cross(f, [0., 1., 0.]));
    let u = cross(s, f);
    [s[0], u[0], -f[0], 0., s[1], u[1], -f[1], 0., s[2], u[2], -f[2], 0., -dot(s, eye), -dot(u, eye), dot(f, eye), 1.]
}
pub fn translate(v: V3) -> M4 { [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., v[0], v[1], v[2], 1.] }
pub fn rot_y(a: f32) -> M4 { let (s, c) = a.sin_cos(); [c, 0., -s, 0., 0., 1., 0., 0., s, 0., c, 0., 0., 0., 0., 1.] }
pub fn rot_x(a: f32) -> M4 { let (s, c) = a.sin_cos(); [1., 0., 0., 0., 0., c, s, 0., 0., -s, c, 0., 0., 0., 0., 1.] }
pub fn scale(k: f32) -> M4 { [k, 0., 0., 0., 0., k, 0., 0., 0., 0., k, 0., 0., 0., 0., 1.] }

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameU { pub vp: M4, pub res: [f32; 4], pub bg: [f32; 4], pub a1: [f32; 4], pub a2: [f32; 4], pub ink: [f32; 4], pub veil: [f32; 4] }

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SlabU { pub mvp: M4, pub model: M4, pub cam: [f32; 4], pub p: [f32; 4], pub size: [f32; 4], pub rim: [f32; 4] }

/// One frame's worth of things to draw, built by the scene.
pub struct Draw {
    pub frame: FrameU,
    /// (slab, uniforms, dust particles), back to front
    pub slabs: Vec<(usize, SlabU, u32)>,
    /// tesseract edges: xyz+w of both ends
    pub lines: Vec<[f32; 8]>,
    pub veil: bool,
}

struct Slab { buf: Buffer, tex: Texture, bind: BindGroup }
struct Target { color: Texture, view: TextureView, depth: TextureView, over: Texture, over_view: TextureView, w: u32, h: u32 }

pub struct Stage {
    device: Device,
    queue: Queue,
    fog: RenderPipeline,
    card: RenderPipeline,
    dust: RenderPipeline,
    lines: RenderPipeline,
    veil: RenderPipeline,
    frame_buf: Buffer,
    frame_bind: BindGroup,
    slab_bgl: BindGroupLayout,
    sampler: Sampler,
    line_buf: Buffer,
    slabs: Vec<Option<Slab>>,
    target: Option<Target>,
}

const FMT: TextureFormat = TextureFormat::Rgba8Unorm;
const DEPTH: TextureFormat = TextureFormat::Depth32Float;
const MAX_LINES: u64 = 64;
const COMMON: &str = include_str!("shaders/common.wgsl");

impl Stage {
    pub fn new(device: Device, queue: Queue) -> Stage {
        let shader = |name: &str, src: &str| device.create_shader_module(ShaderModuleDescriptor {
            label: Some(name), source: ShaderSource::Wgsl(format!("{COMMON}\n{src}").into()),
        });
        let vis = ShaderStages::VERTEX | ShaderStages::FRAGMENT;
        let uniform = |binding| BindGroupLayoutEntry { binding, visibility: vis, ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let frame_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: Some("frame"), entries: &[uniform(0)] });
        let slab_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: Some("slab"), entries: &[
            uniform(0),
            BindGroupLayoutEntry { binding: 1, visibility: vis, ty: BindingType::Texture { sample_type: TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false }, count: None },
            BindGroupLayoutEntry { binding: 2, visibility: vis, ty: BindingType::Sampler(SamplerBindingType::Filtering), count: None },
        ] });
        let layout = |bgls: &[Option<&BindGroupLayout>]| device.create_pipeline_layout(&PipelineLayoutDescriptor { label: None, bind_group_layouts: bgls, immediate_size: 0 });
        let add_blend = BlendState { color: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
            alpha: BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::One, operation: BlendOperation::Add } };
        let depth = |write: bool, cmp: CompareFunction| Some(DepthStencilState { format: DEPTH, depth_write_enabled: Some(write), depth_compare: Some(cmp), stencil: Default::default(), bias: Default::default() });
        let line_attrs = vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        let pipe = |name: &str, src: &str, lay: &PipelineLayout, blend: Option<BlendState>, ds: Option<DepthStencilState>, instanced: bool| {
            let m = shader(name, src);
            let vb = [Some(VertexBufferLayout { array_stride: 32, step_mode: VertexStepMode::Instance, attributes: &line_attrs })];
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(lay),
                vertex: VertexState { module: &m, entry_point: Some("vs"), compilation_options: Default::default(), buffers: if instanced { &vb } else { &[] } },
                primitive: PrimitiveState::default(),
                depth_stencil: ds,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState { module: &m, entry_point: Some("fs"), compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState { format: FMT, blend, write_mask: ColorWrites::ALL })] }),
                multiview_mask: None,
                cache: None,
            })
        };
        let lf = layout(&[Some(&frame_bgl)]);
        let ls = layout(&[Some(&slab_bgl)]);
        let lsf = layout(&[Some(&slab_bgl), Some(&frame_bgl)]);
        let fog = pipe("fog", include_str!("shaders/fog.wgsl"), &lf, None, depth(false, CompareFunction::Always), false);
        let card = pipe("card", include_str!("shaders/card.wgsl"), &ls, Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING), depth(true, CompareFunction::Less), false);
        let dust = pipe("dust", include_str!("shaders/dust.wgsl"), &lsf, Some(add_blend), depth(false, CompareFunction::Less), false);
        let lines = pipe("lines", include_str!("shaders/lines.wgsl"), &lf, Some(add_blend), depth(false, CompareFunction::Less), true);
        let veil = pipe("veil", include_str!("shaders/veil.wgsl"), &lf, None, None, false);
        let frame_buf = device.create_buffer(&BufferDescriptor { label: Some("frame"), size: std::mem::size_of::<FrameU>() as u64, usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, mapped_at_creation: false });
        let frame_bind = device.create_bind_group(&BindGroupDescriptor { label: None, layout: &frame_bgl, entries: &[BindGroupEntry { binding: 0, resource: frame_buf.as_entire_binding() }] });
        let line_buf = device.create_buffer(&BufferDescriptor { label: Some("lines"), size: MAX_LINES * 32, usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&SamplerDescriptor { mag_filter: FilterMode::Linear, min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge, address_mode_v: AddressMode::ClampToEdge, ..Default::default() });
        Stage { device, queue, fog, card, dust, lines, veil, frame_buf, frame_bind, slab_bgl, sampler, line_buf, slabs: Vec::new(), target: None }
    }


    /// Upload a CPU-painted face (premultiplied RGBA) to slab `i`.
    pub fn set_slab(&mut self, i: usize, px: &tiny_skia::Pixmap) {
        let (w, h) = (px.width(), px.height());
        if self.slabs.len() <= i { self.slabs.resize_with(i + 1, || None); }
        let fits = matches!(&self.slabs[i], Some(s) if s.tex.width() == w && s.tex.height() == h);
        if !fits {
            let tex = self.device.create_texture(&TextureDescriptor { label: Some("slab"), size: Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format: FMT,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, view_formats: &[] });
            let buf = match self.slabs[i].take() { Some(s) => s.buf, None => self.device.create_buffer(&BufferDescriptor { label: Some("slab"),
                size: std::mem::size_of::<SlabU>() as u64, usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, mapped_at_creation: false }) };
            let view = tex.create_view(&Default::default());
            let bind = self.device.create_bind_group(&BindGroupDescriptor { label: None, layout: &self.slab_bgl, entries: &[
                BindGroupEntry { binding: 0, resource: buf.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&view) },
                BindGroupEntry { binding: 2, resource: BindingResource::Sampler(&self.sampler) },
            ] });
            self.slabs[i] = Some(Slab { buf, tex, bind });
        }
        let s = self.slabs[i].as_ref().unwrap();
        self.queue.write_texture(TexelCopyTextureInfo { texture: &s.tex, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All },
            px.data(), TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: None },
            Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    }

    /// Draw one frame at w×h. Returns the (stage, overlay) textures when they were (re)created, so the UI can
    /// point its images at them.
    pub fn render(&mut self, w: u32, h: u32, d: &Draw) -> Option<(Texture, Texture)> {
        let (w, h) = (w.max(16), h.max(16));
        let mut fresh = None;
        if self.target.as_ref().is_none_or(|t| t.w != w || t.h != h) {
            let tex = |label, format, usage| self.device.create_texture(&TextureDescriptor { label: Some(label), size: Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format, usage, view_formats: &[] });
            let shown = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
            let color = tex("stage", FMT, shown);
            let over = tex("overlay", FMT, shown);
            let depth = tex("depth", DEPTH, TextureUsages::RENDER_ATTACHMENT).create_view(&Default::default());
            let t = Target { view: color.create_view(&Default::default()), over_view: over.create_view(&Default::default()), color, over, depth, w, h };
            fresh = Some((t.color.clone(), t.over.clone()));
            self.target = Some(t);
        }
        let t = self.target.as_ref().unwrap();
        self.queue.write_buffer(&self.frame_buf, 0, bytemuck::bytes_of(&d.frame));
        let n_lines = d.lines.len().min(MAX_LINES as usize);
        if n_lines > 0 { self.queue.write_buffer(&self.line_buf, 0, bytemuck::cast_slice(&d.lines[..n_lines])); }
        for (i, u, _) in &d.slabs {
            if let Some(Some(s)) = self.slabs.get(*i) { self.queue.write_buffer(&s.buf, 0, bytemuck::bytes_of(u)); }
        }
        let mut enc = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut p = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some("stage"),
                color_attachments: &[Some(RenderPassColorAttachment { view: &t.view, depth_slice: None, resolve_target: None,
                    ops: Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store } })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment { view: &t.depth,
                    depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Discard }), stencil_ops: None }),
                ..Default::default()
            });
            p.set_pipeline(&self.fog);
            p.set_bind_group(0, &self.frame_bind, &[]);
            p.draw(0..3, 0..1);
            // glass first (depth written), then the additive light that should sit among it
            p.set_pipeline(&self.card);
            for (i, _, _) in &d.slabs {
                if let Some(Some(s)) = self.slabs.get(*i) { p.set_bind_group(0, &s.bind, &[]); p.draw(0..6, 0..1); }
            }
            if n_lines > 0 {
                p.set_pipeline(&self.lines);
                p.set_bind_group(0, &self.frame_bind, &[]);
                p.set_vertex_buffer(0, self.line_buf.slice(..));
                p.draw(0..6, 0..n_lines as u32);
            }
            p.set_pipeline(&self.dust);
            p.set_bind_group(1, &self.frame_bind, &[]);
            for (i, _, n) in &d.slabs {
                if *n == 0 { continue; }
                if let Some(Some(s)) = self.slabs.get(*i) { p.set_bind_group(0, &s.bind, &[]); p.draw(0..6, 0..*n); }
            }
        }
        if d.veil {
            let mut p = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some("veil"),
                color_attachments: &[Some(RenderPassColorAttachment { view: &t.over_view, depth_slice: None, resolve_target: None,
                    ops: Operations { load: LoadOp::Clear(Color::TRANSPARENT), store: StoreOp::Store } })],
                ..Default::default()
            });
            p.set_pipeline(&self.veil);
            p.set_bind_group(0, &self.frame_bind, &[]);
            p.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
        fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_maps_the_target_to_the_centre_of_the_screen() {
        let vp = mul(&perspective(0.7, 1.6, 0.1, 100.), &look_at([3., 1., 8.], [0., 0., 0.]));
        let p = [0f32, 0., 0., 1.];
        let c: Vec<f32> = (0..4).map(|r| (0..4).map(|k| vp[k * 4 + r] * p[k]).sum()).collect();
        assert!((c[0] / c[3]).abs() < 1e-4 && (c[1] / c[3]).abs() < 1e-4);
        assert!(c[2] / c[3] > 0. && c[2] / c[3] < 1., "depth in 0..1");
    }
}
