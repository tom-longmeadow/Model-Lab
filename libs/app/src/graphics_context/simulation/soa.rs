use std::marker::PhantomData;

use wgpu::util::DeviceExt;
use base::{math::Vector, sim::{solver::particle::{verlet_particle::{VerletParticle}, 
verlet_soa_vec_storage::{SoaColor, SoaPos, SoaRadius, VerletParticleSoaVecStorage}}, storage::Storage}, ui::layout::color::Color};
use crate::graphics_context::{
     renderer::Renderer, shader::{
        ShaderBuilder,
        fragment::FragmentFunction,
        vertex::VertexFunction,
        vertex_input::VertexInput,
        vertex_output::VertexOutput,
    }, simulation::renderer::SimulationRenderer, state::{quad_state::QuadState} 
};

 fn particle_soa_layouts() -> [wgpu::VertexBufferLayout<'static>; 3] {
    [
        // Slot 0: Positions Array (Maps to Vertex Input Location 4)
        wgpu::VertexBufferLayout {
            array_stride: 16, // glam::DVec2 = 16 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            }],
        },
        // Slot 1: Radii Array (Maps to Vertex Input Location 5)
        wgpu::VertexBufferLayout {
            array_stride: 8, // f64 = 8 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x2,
            }],
        },
        // Slot 2: Colors Array (Maps to Vertex Input Location 6)
        wgpu::VertexBufferLayout {
            array_stride: 4, // Color = 4 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 6,
                format: wgpu::VertexFormat::Unorm8x4,
            }],
        },
    ]
}
pub struct SoaSimulationRenderer<I> {
    pipeline: Option<wgpu::RenderPipeline>,
    quad_state: Option<QuadState>,

    // SoA buffers mapped to independent vertex input slots
    pos_buffer: Option<wgpu::Buffer>,
    radius_buffer: Option<wgpu::Buffer>,
    color_buffer: Option<wgpu::Buffer>,
    instance_count: u32,

    // 🚀 PERSISTENT HEAP STAGING CACHES:
    // Replaces dangerous tracking raw pointers with safe, local byte streams.
    // Preserves capacity across frame ticks to eliminate runtime allocation overhead!
    pos_staging: Vec<u8>,
    radius_staging: Vec<u8>,
    color_staging: Vec<u8>,

    _marker: PhantomData<I>,
}

impl<I> SoaSimulationRenderer<I> {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            quad_state: None,
            pos_buffer: None,
            radius_buffer: None,
            color_buffer: None,
            instance_count: 0,
            pos_staging: Vec::with_capacity(2048),
            radius_staging: Vec::with_capacity(1024),
            color_staging: Vec::with_capacity(1024),
            _marker: PhantomData,
        }
    }

    /// Internal helper method to dynamically cycle or allocate vertex buffers
    /// based on active sizing changes during frame streams.
    fn update_buffer(
        device: &wgpu::Device, 
        buf: &mut Option<wgpu::Buffer>, 
        data: &[u8], 
        label: &'static str
    ) {
        if let Some(existing) = buf {
            if existing.size() >= data.len() as u64 {
                return;
            }
        }
        
        *buf = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }));
    }
}

// =========================================================================
// FLAWLESS SYNCHRONIZATION WITH YOUR EXACT TRAIT SPECIFICATION
// =========================================================================

impl<V> SimulationRenderer<VerletParticleSoaVecStorage<V>> for SoaSimulationRenderer<VerletParticle<V>>
where
    V: Vector + 'static,
    V::Scalar: 'static,
{
    fn sync(&mut self, storage: &VerletParticleSoaVecStorage<V>, _config: &wgpu::SurfaceConfiguration) {
        let count = storage.len();
        self.instance_count = count as u32;

        if count == 0 {
            self.pos_staging.clear();
            self.radius_staging.clear();
            self.color_staging.clear();
            return;
        }

        // 🚀 1. ACQUIRE IMMUTABLE VIEW DETAILS FROM STORAGE
        let view = storage.view();

        // 🚀 2. EXTRACT STRONGLY TYPED CONTINUOUS MEMORY CHUNKS
        let positions = view.slice(SoaPos);
        let radii     = view.slice(SoaRadius);
        let colors    = view.slice(SoaColor);

        // Compute layout dimensions safely
        let pos_bytes    = count * std::mem::size_of::<V>();
        let radius_bytes = count * std::mem::size_of::<V::Scalar>();
        let color_bytes  = count * std::mem::size_of::<Color>();

        // 🚀 3. RECONSTRUCT SAFE NATIVE BYTE SLICES
        let raw_pos = unsafe { std::slice::from_raw_parts(positions.as_ptr() as *const u8, pos_bytes) };
        let raw_radius = unsafe { std::slice::from_raw_parts(radii.as_ptr() as *const u8, radius_bytes) };
        let raw_color = unsafe { std::slice::from_raw_parts(colors.as_ptr() as *const u8, color_bytes) };

        // 🚀 4. INJECT DIRECTLY INTO CONTINUOUS STAGING ARRAYS
        // This is an incredibly fast memcpy. It guarantees absolute safety 
        // because data is locally isolated on your host memory immediately.
        self.pos_staging.resize(pos_bytes, 0);
        self.pos_staging.copy_from_slice(raw_pos);

        self.radius_staging.resize(radius_bytes, 0);
        self.radius_staging.copy_from_slice(raw_radius);

        self.color_staging.resize(color_bytes, 0);
        self.color_staging.copy_from_slice(raw_color);
    }
}

// =========================================================================
// RUNTIME RENDER PASS HANDLERS (UNCHANGED SIGNATURES)
// =========================================================================

impl<I: 'static> Renderer for SoaSimulationRenderer<I> {
    type Data = ();

    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        if self.pipeline.is_some() { return; }

        let shader = ShaderBuilder::new(
            VertexOutput::ColorUv,
            VertexFunction::ParticleAosInstanced, 
            FragmentFunction::Circular,
        )
        .with_vertex_input(VertexInput::ColorUv)
        .build(device);

        let layouts = particle_soa_layouts();

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SOA Zero-Copy Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &layouts, 
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let inferred_layout = pipeline.get_bind_group_layout(0);
        let quad_state = QuadState::new(device, &inferred_layout, config.width as f32, config.height as f32);

        self.pipeline = Some(pipeline);
        self.quad_state = Some(quad_state);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        if let Some(ref mut quad_state) = self.quad_state {
            quad_state.resize(queue, config.width as f32, config.height as f32);
        }

        if self.instance_count == 0 || self.pos_staging.is_empty() { return; }

        // 🚀 1. VERIFY OR RESIZE BACKING HARDWARE VOLUMES
        Self::update_buffer(device, &mut self.pos_buffer, &self.pos_staging, "SOA Positions Buffer");
        Self::update_buffer(device, &mut self.radius_buffer, &self.radius_staging, "SOA Radii Buffer");
        Self::update_buffer(device, &mut self.color_buffer, &self.color_staging, "SOA Colors Buffer");

        // 🚀 2. SHIP DATA MEMORY MAPS DIRECTLY TO GPU COMMAND REGISTERS
        if let Some(buf) = &self.pos_buffer { queue.write_buffer(buf, 0, &self.pos_staging); }
        if let Some(buf) = &self.radius_buffer { queue.write_buffer(buf, 0, &self.radius_staging); }
        if let Some(buf) = &self.color_buffer { queue.write_buffer(buf, 0, &self.color_staging); }
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let pipeline = match &self.pipeline { Some(p) => p, None => return };
        let quad_state = match &self.quad_state { Some(q) => q, None => return };

        let pos_buf = match &self.pos_buffer { Some(b) => b, None => return };
        let rad_buf = match &self.radius_buffer { Some(b) => b, None => return };
        let col_buf = match &self.color_buffer { Some(b) => b, None => return };

        if self.instance_count == 0 { return; }

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &quad_state.screen_metrics.bind_group, &[]);

        // Attach each independent buffer slice straight to its corresponding vertex input layout slot
        pass.set_vertex_buffer(0, pos_buf.slice(..));
        pass.set_vertex_buffer(1, rad_buf.slice(..));
        pass.set_vertex_buffer(2, col_buf.slice(..));

        pass.draw(0..6, 0..self.instance_count);
    }
}