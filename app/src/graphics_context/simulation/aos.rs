use wgpu::util::DeviceExt;
use base::sim::storage::AosCpuStorage;
use crate::graphics_context::{
    renderer::Renderer, shader::{
        ShaderBuilder,
        fragment::FragmentFunction,
        vertex::VertexFunction,
        vertex_input::VertexInput,
        vertex_output::VertexOutput,
    }, simulation::renderer::SimulationRenderer, vertex::GpuVertex
};

/// A renderer for ANY AoS simulation data.
///
/// Each item is rendered as a colored quad in NDC space. The `to_vertex` closure
/// maps an item to a center `GpuVertex` (position in NDC, color). The renderer
/// expands that into 6 vertices (2 triangles) forming a `quad_size`-radius square.
///
/// Uses the NDC passthrough vertex shader — positions are in [-1, 1] clip space.
pub struct AosSimulationRenderer<I> {
    to_vertex: Box<dyn Fn(&I) -> GpuVertex>,
    quad_size: f32,
    /// Persistent CPU-side staging buffer. Grown as needed, never shrunk.
    /// Eliminates a per-frame heap allocation.
    staged: Vec<GpuVertex>,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
}

impl<I> AosSimulationRenderer<I> {
    /// Create a new renderer.
    ///
    /// - `initial_data`: snapshot of simulation items
    /// - `to_vertex`: maps an item to a center `GpuVertex` (position = NDC, color = particle color)
    /// - `quad_size`: half-size of the rendered quad in NDC units (e.g. `0.05` = visible square)
    pub fn new(
        to_vertex: impl Fn(&I) -> GpuVertex + 'static,
        quad_size: f32,
    ) -> Self {
        Self {
            to_vertex: Box::new(to_vertex),
            quad_size,
            staged: Vec::new(),
            pipeline: None,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }
}

impl<I: 'static> Renderer for AosSimulationRenderer<I> {
    type Data = ();

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipeline.is_some() {
            return;
        }

        // NDC passthrough vertex + plain color fragment.
        // The Circular fragment shader uses `point_coord` which is only valid for PointList.
        let shader = ShaderBuilder::new(
            VertexOutput::ColorUv,
            VertexFunction::ParticleAosColor,
            FragmentFunction::Circular,
        )
        .with_vertex_input(VertexInput::ColorUv)
        .label("AOS Simulation Quad Shader")
        .build(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("AOS Simulation Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("AOS Simulation Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GpuVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
    }

    fn update_data(&mut self, _data: Self::Data) {}

    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
    ) {
        if self.staged.is_empty() {
            self.vertex_count = 0;
            return;
        }

        self.vertex_count = self.staged.len() as u32;
        let required_size = (self.staged.len() * std::mem::size_of::<GpuVertex>()) as u64;
        if let Some(buf) = &self.vertex_buffer {
            if buf.size() >= required_size {
                queue.write_buffer(buf, 0, bytemuck::cast_slice(&self.staged));
                return;
            }
        }
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("AOS Simulation Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.staged),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }));
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 || self.vertex_buffer.is_none() || self.pipeline.is_none() {
            return;
        }
        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

/// Implement `SimulationRenderer` generically for any `AosCpuStorage` whose item matches `I`.
/// This means `AosSimulationRenderer<Particle>` works with `VecStorage`, or any other
/// AoS storage that holds `Particle` — without needing a new renderer type.
impl<I: Clone + 'static, S: AosCpuStorage<Item = I>> SimulationRenderer<S> for AosSimulationRenderer<I> {
    /// Expand simulation storage directly into the GPU staging buffer.
    /// No intermediate `Vec<I>` clone — `self.staged` is reused across frames.
    fn sync(&mut self, storage: &S, config: &wgpu::SurfaceConfiguration) {
        let items = storage.as_slice();
        let aspect = config.height as f32 / config.width as f32;
        let sx = self.quad_size * aspect;
        let sy = self.quad_size;

        self.staged.clear();
        self.staged.reserve(items.len() * 6);
        for item in items {
            let center = (self.to_vertex)(item);
            let [x, y, z] = center.position;
            let c = center.color;
            let n = [0.0_f32, 0.0, 1.0];
            self.staged.push(GpuVertex { position: [x - sx, y - sy, z], normal: n, uv: [0.0, 0.0], color: c });
            self.staged.push(GpuVertex { position: [x + sx, y - sy, z], normal: n, uv: [1.0, 0.0], color: c });
            self.staged.push(GpuVertex { position: [x + sx, y + sy, z], normal: n, uv: [1.0, 1.0], color: c });
            self.staged.push(GpuVertex { position: [x - sx, y - sy, z], normal: n, uv: [0.0, 0.0], color: c });
            self.staged.push(GpuVertex { position: [x + sx, y + sy, z], normal: n, uv: [1.0, 1.0], color: c });
            self.staged.push(GpuVertex { position: [x - sx, y + sy, z], normal: n, uv: [0.0, 1.0], color: c });
        }
    }
}

 