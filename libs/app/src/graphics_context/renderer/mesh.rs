use base::mesh::{Mesh, kind::MeshKind, vertex::Vertex};
use wgpu::util::DeviceExt;
use crate::graphics_context::{
    buffers::GpuMeshBuffers, renderer::Renderer, shader::{ShaderBuilder, 
        fragment::FragmentFunction, vertex::VertexFunction, vertex_input::VertexInput, vertex_output::VertexOutput}, vertex::WgpuVertexExt,  
};

 
pub struct MeshRenderer {
    data: Vec<Mesh>,
    pipeline_tri: Option<wgpu::RenderPipeline>,
    pipeline_line: Option<wgpu::RenderPipeline>,
    uniform_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    buffers: Vec<GpuMeshBuffers>,
}

impl MeshRenderer {
    pub fn new(data: Vec<Mesh>) -> Self {
        Self {
            data,
            pipeline_tri: None,
            pipeline_line: None,
            uniform_buffer: None,
            bind_group: None,
            buffers: Vec::new(),
        }
    }
}

impl Renderer for MeshRenderer {
    type Data = Vec<Mesh>;

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipeline_tri.is_some() {
            return;
        }

        // 1) Uniform buffer for screen size
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MeshRenderer Uniform Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 2) Bind group layout + bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("MeshRenderer BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MeshRenderer BG"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

      // 3) Pipelines
        // Replace the old shader creation with the new ShaderBuilder
        let shader = ShaderBuilder::new(
            VertexOutput::Color,
            VertexFunction::Ui,
            FragmentFunction::Passthrough,
        )
        .label("UI Mesh Shader")
        .with_vertex_input(VertexInput::Color)
        .build(device);
    
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("MeshRenderer Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)], // Wrap in Some()
            immediate_size: 0, // Add immediate_size field
        });

        self.pipeline_tri = Some(build_pipeline(
            device,
            &pipeline_layout,
            &shader,
            config.format,
            wgpu::PrimitiveTopology::TriangleList,
        ));

        self.pipeline_line = Some(build_pipeline(
            device,
            &pipeline_layout,
            &shader,
            config.format,
            wgpu::PrimitiveTopology::LineList,
        ));

        self.uniform_buffer = Some(uniform_buffer);
        self.bind_group = Some(bind_group);
    }

    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration, 
    ) {
        if let Some(buf) = &self.uniform_buffer {
            let screen_size = [config.width as f32, config.height as f32];
            queue.write_buffer(buf, 0, bytemuck::cast_slice(&screen_size));
        }

        self.buffers = self.data
            .iter()
            .map(|mesh| {
                // 1. Manually unpack and convert fields directly inside a clean vector map
                let gpu_verts: Vec<Vertex> = mesh.vertices
                    .iter()
                    .map(|v| Vertex {
                        position: v.position.into(), // Converts Vec3 -> [f32; 3]
                        normal:   v.normal.into(),   // Converts Vec3 -> [f32; 3]
                        uv:       v.uv.into(),       // Converts Vec2 -> [f32; 2]
                        color:    v.color.into(),    // Converts Vec4 -> [f32; 4]
                    })
                    .collect();

                // 2. Initialize vertex buffer
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("MeshRenderer Vertex Buffer"),
                    contents: bytemuck::cast_slice(&gpu_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                // 3. Initialize index buffer
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("MeshRenderer Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                GpuMeshBuffers {
                    vertex_buffer,
                    index_buffer,
                    index_count: mesh.indices.len() as u32,
                    kind: mesh.kind,  
                }
            })
            .collect();
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.pipeline_tri.is_none() || self.bind_group.is_none() {
            return;
        }

        pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        for buf in &self.buffers {
            let pipeline = match buf.kind {
                MeshKind::Triangle => self.pipeline_tri.as_ref().unwrap(),
                MeshKind::Line => self.pipeline_line.as_ref().unwrap(),
                MeshKind::Point => continue,
            };
            pass.set_pipeline(pipeline);
            pass.set_vertex_buffer(0, buf.vertex_buffer.slice(..));
            pass.set_index_buffer(buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..buf.index_count, 0, 0..1);
        }
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("MeshRenderer Pipeline"), // Wrap label in Some()
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[<base::mesh::vertex::Vertex as WgpuVertexExt>::layout()],
            compilation_options: Default::default(), // Add compilation_options
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(), // Add compilation_options
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,       // Add cache
    })
}
 