 use wgpu::util::DeviceExt;
use std::slice;
use base::{
    math::DVec2,
    sim::{solver::particle::{verlet_particle::VerletParticle, verlet_soa_vec_storage::VerletCol}, storage::SoaCpuStorage}, 
};
use crate::graphics_context::{
    renderer::Renderer, shader::{
        ShaderBuilder, fragment::FragmentFunction, vertex::VertexFunction,
        vertex_input::VertexInput, vertex_output::VertexOutput,
    }, simulation::renderer::SimulationRenderer, vertex::GpuVertex
};

pub type MyVerletParticle = VerletParticle<DVec2>;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniforms {
    aspect_ratio: f32,
    screen_width: f32,
    screen_height: f32,
    _padding: f32,
}

// STRATEGY: Create discrete wgpu::VertexBufferLayouts for each tracking stream column
fn soa_position_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<DVec2>() as wgpu::BufferAddress, // 16 bytes
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 4, // Matches your existing vs shader location
            format: wgpu::VertexFormat::Float32x4,
        }],
    }
}

fn soa_radius_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<f64>() as wgpu::BufferAddress, // 8 bytes
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 5, // Matches your existing vs shader location
            format: wgpu::VertexFormat::Float32x2,
        }],
    }
}

fn soa_color_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 4 as wgpu::BufferAddress, // 4 bytes (RGBA packed unorm)
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 6, // Matches your existing vs shader location
            format: wgpu::VertexFormat::Unorm8x4,
        }],
    }
}

pub struct SoaSimulationRenderer<I> {
    pipeline: Option<wgpu::RenderPipeline>,
    unit_quad_buffer: Option<wgpu::Buffer>, 
    
    // Independent discrete vertex buffers allocated per tracked data layout column
    position_buffer: Option<wgpu::Buffer>,
    radius_buffer: Option<wgpu::Buffer>,
    color_buffer: Option<wgpu::Buffer>,
    instance_count: u32, 

    aspect_buffer: Option<wgpu::Buffer>,
    aspect_bind_group: Option<wgpu::BindGroup>,

    // Tracks individual raw component pointer boundaries across passes
    pos_ptr: *const u8,
    pos_bytes: usize,
    
    radius_ptr: *const u8,
    radius_bytes: usize,
    
    color_ptr: *const u8,
    color_bytes: usize,
    
    _marker: std::marker::PhantomData<I>,
}

impl<I> SoaSimulationRenderer<I> {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            unit_quad_buffer: None,
            position_buffer: None,
            radius_buffer: None,
            color_buffer: None,
            instance_count: 0, 
            aspect_buffer: None,
            aspect_bind_group: None,
            pos_ptr: std::ptr::null(),
            pos_bytes: 0,
            radius_ptr: std::ptr::null(),
            radius_bytes: 0,
            color_ptr: std::ptr::null(),
            color_bytes: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

// FIXED: Now safely implements SimulationRenderer targeting your SoA container columns layout
impl<S: SoaCpuStorage<Layout = MyVerletParticle>> SimulationRenderer<S> for SoaSimulationRenderer<MyVerletParticle> {
    fn sync(&mut self, storage: &S, _config: &wgpu::SurfaceConfiguration) {
        let len = storage.len();
        self.instance_count = len as u32;

        if len == 0 {
            self.pos_ptr = std::ptr::null();
            self.radius_ptr = std::ptr::null();
            self.color_ptr = std::ptr::null();
            self.pos_bytes = 0;
            self.radius_bytes = 0;
            self.color_bytes = 0;
            return;
        }

        // Access the low-level RawColumns memory regions using your Enum layout variants
        let cols = storage.columns();
        
        let p_col = &cols[VerletCol::Pos as usize];
        self.pos_ptr = p_col.ptr;
        self.pos_bytes = len * std::mem::size_of::<DVec2>();

        let r_col = &cols[VerletCol::Radius as usize];
        self.radius_ptr = r_col.ptr;
        self.radius_bytes = len * std::mem::size_of::<f64>();

        let c_col = &cols[VerletCol::Color as usize];
        self.color_ptr = c_col.ptr;
        self.color_bytes = len * 4; // Size of a packed structural RGBA Color
    }
}

impl<I: 'static> Renderer for SoaSimulationRenderer<I> {
    type Data = ();

    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        if self.pipeline.is_some() { return; }

        let unit_quad_vertices = [
            GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
            GpuVertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0, -1.0], color: [1.0; 4] },
            GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
            GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
            GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
            GpuVertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0,  1.0], color: [1.0; 4] },
        ];

        self.unit_quad_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SoA Unit Quad Buffer"),
            contents: bytemuck::cast_slice(&unit_quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        let aspect_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SoA Aspect Ratio Uniform Bind Group Layout"),
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

        let initial_metrics = ScreenUniforms {
            aspect_ratio: config.width as f32 / config.height as f32,
            screen_width: config.width as f32,
            screen_height: config.height as f32,
            _padding: 0.0,
        };

        let aspect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SoA Screen Metrics Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_metrics),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let aspect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SoA Aspect Ratio Uniform Bind Group"),
            layout: &aspect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: aspect_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SoA Zero-Copy Layout"),
            bind_group_layouts: &[Some(&aspect_bind_group_layout)], 
            immediate_size: 0,
        });

        self.aspect_buffer = Some(aspect_buffer);
        self.aspect_bind_group = Some(aspect_bind_group);

        // Your existing WGSL shader remains completely compatible because the shader location IDs match!
        let shader = ShaderBuilder::new(
            VertexOutput::ColorUv,
            VertexFunction::ParticleAosInstanced,
            FragmentFunction::Circular,
        )
        .with_vertex_input(VertexInput::ColorUv)
        .build(device);

        self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SoA Zero-Copy Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // FIXED: Bind all 3 parallel attribute buffer layouts to their respective sequential slots
                buffers: &[
                    GpuVertex::layout(), 
                    soa_position_layout(), 
                    soa_radius_layout(), 
                    soa_color_layout()
                ],
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
        }));
    }
 fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, _config: &wgpu::SurfaceConfiguration) {
        if self.instance_count == 0 { return; }

        // Helper macro to reuse or scale dynamic column buffers safely
        macro_rules! update_column_buffer {
            ($buf_field:expr, $ptr:expr, $bytes:expr, $label:expr) => {
                if $bytes > 0 && !$ptr.is_null() {
                    let raw_slice = unsafe { slice::from_raw_parts($ptr, $bytes) };
                    
                    // Check if we need to completely allocate or grow our GPU buffer
                    let needs_realloc = match &$buf_field {
                        Some(b) => b.size() < $bytes as wgpu::BufferAddress,
                        None => true,
                    };

                    if needs_realloc {
                        $buf_field = Some(device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some($label),
                            size: $bytes as wgpu::BufferAddress,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }));
                    }

                    // Upload bytes directly over the pipeline timeline via non-blocking commands
                    if let Some(b) = &$buf_field {
                        queue.write_buffer(b, 0, raw_slice);
                    }
                } else {
                    $buf_field = None;
                }
            };
        }

        // Stream each data column vector straight to parallel GPU channels
        update_column_buffer!(self.position_buffer, self.pos_ptr, self.pos_bytes, "SoA Position Buffer");
        update_column_buffer!(self.radius_buffer, self.radius_ptr, self.radius_bytes, "SoA Radius Buffer");
        update_column_buffer!(self.color_buffer, self.color_ptr, self.color_bytes, "SoA Color Buffer");
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 { return; }
        
        let pipeline = match &self.pipeline { Some(p) => p, None => return };
        let quad_buf = match &self.unit_quad_buffer { Some(b) => b, None => return };
        let pos_buf = match &self.position_buffer { Some(b) => b, None => return };
        let rad_buf = match &self.radius_buffer { Some(b) => b, None => return };
        let col_buf = match &self.color_buffer { Some(b) => b, None => return };
        let bind_group = match &self.aspect_bind_group { Some(bg) => bg, None => return };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        
        // Map individual parallel attribute columns to separate data slots
         render_pass.set_vertex_buffer(0, quad_buf.slice(..));
        render_pass.set_vertex_buffer(1, pos_buf.slice(..));
        render_pass.set_vertex_buffer(2, rad_buf.slice(..));
        render_pass.set_vertex_buffer(3, col_buf.slice(..));

        render_pass.draw(0..6, 0..self.instance_count);
    }
}