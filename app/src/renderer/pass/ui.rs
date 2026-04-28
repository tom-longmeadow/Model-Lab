
use base::mesh::{Mesh, kind::MeshKind};
use wgpu::util::DeviceExt;

use crate::renderer::{buffers::GpuMeshBuffers, pass::RenderPass, vertex::GpuVertex};

pub struct UiMeshRenderPass {
    meshes: Vec<Mesh>,
    buffers: Vec<GpuMeshBuffers>,
    pipeline_tri: Option<wgpu::RenderPipeline>,
    pipeline_line: Option<wgpu::RenderPipeline>,
    uniform_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    screen_size: [f32; 2],
}

impl UiMeshRenderPass {
    pub fn new(meshes: Vec<Mesh>, screen_width: f32, screen_height: f32) -> Self {
        Self {
            meshes,
            buffers: Vec::new(),
            pipeline_tri: None,
            pipeline_line: None,
            uniform_buffer: None,
            bind_group: None,
            screen_size: [screen_width, screen_height],
        }
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = [width, height];
    }
}

impl RenderPass for UiMeshRenderPass {
    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {

        // println!("UiMeshRenderPass::prepare — meshes: {}", self.meshes.len());
        // for m in &self.meshes {
        //     println!("  kind: {:?}  verts: {}  indices: {}", m.mesh_type, m.vertices.len(), m.indices.len());
        // }

        if self.uniform_buffer.is_some() {
            self.screen_size = [config.width as f32, config.height as f32];
            return;
        }

        // 1) uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UiMesh Uniform Buffer"),
            contents: bytemuck::cast_slice(&self.screen_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 2) bind group layout + bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UiMesh BGL"),
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
            label: Some("UiMesh BG"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 3) pipelines
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/ui.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UiMesh Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        self.pipeline_tri = Some(build_pipeline(
            device, &pipeline_layout, &shader, config.format,
            wgpu::PrimitiveTopology::TriangleList,
        ));

        self.pipeline_line = Some(build_pipeline(
            device, &pipeline_layout, &shader, config.format,
            wgpu::PrimitiveTopology::LineList,
        ));

        // 4) upload mesh buffers
        self.buffers = self.meshes.iter().map(|mesh| {
            let gpu_verts: Vec<GpuVertex> = mesh.vertices.iter().copied().map(GpuVertex::from).collect();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UiMesh Vertex Buffer"),
                contents: bytemuck::cast_slice(&gpu_verts),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UiMesh Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            GpuMeshBuffers {
                vertex_buffer,
                index_buffer,
                index_count: mesh.indices.len() as u32,
                kind: mesh.mesh_type,
            }
        }).collect();

        // 5) store
        self.uniform_buffer = Some(uniform_buffer);
        self.bind_group = Some(bind_group);
    }

    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
        for buf in &self.buffers {
            let pipeline = match buf.kind {
                MeshKind::Triangle => self.pipeline_tri.as_ref().unwrap(),
                MeshKind::Line     => self.pipeline_line.as_ref().unwrap(),
                MeshKind::Point    => continue,
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, buf.vertex_buffer.slice(..));
            pass.set_index_buffer(buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..buf.index_count, 0, 0..1);
        }
    }

    fn update(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(buf) = &self.uniform_buffer {
            queue.write_buffer(buf, 0, bytemuck::cast_slice(&self.screen_size));
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
        label: Some("UiMesh Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[GpuVertex::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(), 
        multiview_mask: None,
        cache: None,
    })
}