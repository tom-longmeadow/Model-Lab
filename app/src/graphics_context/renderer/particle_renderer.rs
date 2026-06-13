use wgpu::{Device, Queue, SurfaceConfiguration, util::DeviceExt};
use crate::graphics_context::{
    renderer::Renderer,
    vertex::GpuVertex,
    shader::{
        ShaderBuilder,
        vertex::VertexFunction,
        vertex_input::VertexInput,
        vertex_output::VertexOutput,
        fragment::FragmentFunction,
    },
};
use base::sim::storage::AosCpuStorage;
use impls::simulation::particle::particle_2d::VerletParticle2d;

/// A concrete renderer that knows how to draw particles from an AoS storage.
pub struct ParticleRenderer {
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    particle_count: u32,
}

impl ParticleRenderer {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            vertex_buffer: None,
            particle_count: 0,
        }
    }
}

// This renderer can draw any data `D` that behaves like AoS particle storage.
impl<D> Renderer<D> for ParticleRenderer
where
    D: AosCpuStorage<Item = VerletParticle2d>,
{
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipeline.is_some() { return; }

        // Build the shader using the new modular system.
        // We're using:
        // - VertexInput::Color (position + color)
        // - VertexOutput::Color (clip_position + color)
        // - VertexFunction::ParticleAosColor (AOS particle vertex shader)
        // - FragmentFunction::Circular (draws circular points)
        let shader = ShaderBuilder::new(
            VertexOutput::Color,
            VertexFunction::ParticleAosColor,
            FragmentFunction::Circular,
        )
        .label("AOS Particle Shader")
        .with_vertex_input(VertexInput::Color) // AOS needs the input struct
        .build(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[],
             immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
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
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,       
        });

        self.pipeline = Some(pipeline);
    }

    fn update(&mut self, device: &Device, queue: &Queue, _config: &SurfaceConfiguration, data: &D) {
        let storage_slice = data.as_slice();
        self.particle_count = storage_slice.len() as u32;
        if self.particle_count == 0 { return; }

        let gpu_verts: Vec<GpuVertex> = storage_slice.iter().map(|p| GpuVertex {
            position: [p.pos[0] as f32, p.pos[1] as f32, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: [1.0, 0.5, 0.2, 1.0],
        }).collect();

        // Create or update the vertex buffer
        if let Some(buffer) = &self.vertex_buffer {
            if buffer.size() >= gpu_verts.len() as u64 * std::mem::size_of::<GpuVertex>() as u64 {
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&gpu_verts));
                return;
            }
        }

        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&gpu_verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        );
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.particle_count == 0 || self.vertex_buffer.is_none() {
            return;
        }

        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..self.particle_count, 0..1);
    }
}