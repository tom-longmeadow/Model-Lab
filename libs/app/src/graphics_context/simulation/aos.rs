use impls::simulation::verlet_2d::particle::Particle;
use wgpu::util::DeviceExt;
use base::sim::storage::AosCpuStorage;
use crate::graphics_context::{
    renderer::Renderer, shader::{
        ShaderBuilder,
        fragment::FragmentFunction,
        vertex::VertexFunction,
        vertex_input::VertexInput,
        vertex_output::VertexOutput,
    }, simulation::{renderer::SimulationRenderer}, vertex::GpuVertex
};

// Ensure the struct matches uniform alignment rules (multiples of 16 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniforms {
    aspect_ratio: f32,
    screen_width: f32,
    screen_height: f32,
    _padding: f32, // Pad to 16 bytes
}

fn particle_layout() -> wgpu::VertexBufferLayout<'static> {
   wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Particle>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // 1. pos (glam::DVec2 = 16 bytes). Starts at byte index 0.
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            // Skip pos_old (offset 16) and acc (offset 32)
            
            // 2. radius (f64 = 8 bytes). Starts at byte index 48.
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x2,
            },
            // 3. color (Color = 4 bytes). Starts at byte index 56.
            wgpu::VertexAttribute {
                offset: 56,
                shader_location: 6,
                format: wgpu::VertexFormat::Unorm8x4,
            },
        ],
    }
}

pub struct AosSimulationRenderer<I> {
    pipeline: Option<wgpu::RenderPipeline>,
    unit_quad_buffer: Option<wgpu::Buffer>, 
    instance_buffer: Option<wgpu::Buffer>,
    instance_count: u32, 

    // NEW: Uniform buffer storage for the GPU aspect ratio layout
    aspect_buffer: Option<wgpu::Buffer>,
    aspect_bind_group: Option<wgpu::BindGroup>,

    // Track pointer states safely between sync and update passes
    raw_data_ptr: *const u8,
    raw_data_bytes: usize,
    
    _marker: std::marker::PhantomData<I>,
}

impl<I> AosSimulationRenderer<I> {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            unit_quad_buffer: None,
            instance_buffer: None,
            instance_count: 0, 
             aspect_buffer: None,
            aspect_bind_group: None,
            raw_data_ptr: std::ptr::null(),
            raw_data_bytes: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S: AosCpuStorage<Item = Particle>> SimulationRenderer<S> for AosSimulationRenderer<Particle> {
    fn sync(&mut self, storage: &S, _config: &wgpu::SurfaceConfiguration) {
        
        let items = storage.as_slice();
        self.instance_count = items.len() as u32;

        if items.is_empty() {
            self.raw_data_ptr = std::ptr::null();
            self.raw_data_bytes = 0;
            return;
        } 
        self.raw_data_ptr = items.as_ptr() as *const u8;
        self.raw_data_bytes = items.len() * std::mem::size_of::<Particle>(); 
    }
 
}

impl<I: 'static> Renderer for AosSimulationRenderer<I> {
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
            label: Some("Unit Quad Buffer"),
            contents: bytemuck::cast_slice(&unit_quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
 

        let aspect_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Aspect Ratio Uniform Bind Group Layout"),
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

        // let initial_aspect = config.width as f32 / config.height as f32;
        // let aspect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Aspect Ratio Uniform Buffer"),
        //     contents: bytemuck::bytes_of(&initial_aspect),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });

        let initial_metrics = ScreenUniforms {
            aspect_ratio: config.width as f32 / config.height as f32,
            screen_width: config.width as f32,
            screen_height: config.height as f32,
            _padding: 0.0,
        };

        let aspect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Metrics Uniform Buffer"),
            contents: bytemuck::bytes_of(&initial_metrics), // Ingests the full 16 bytes now
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });


        let aspect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Aspect Ratio Uniform Bind Group"),
            layout: &aspect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: aspect_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("AOS Zero-Copy Layout"),
            // FIX: Wrap the inner item in Some() to match the expected Option<&BindGroupLayout>
            bind_group_layouts: &[Some(&aspect_bind_group_layout)], 
            immediate_size: 0,
        });

         self.aspect_buffer = Some(aspect_buffer);
        self.aspect_bind_group = Some(aspect_bind_group);

         let shader = ShaderBuilder::new(
            VertexOutput::ColorUv,
            VertexFunction::ParticleAosInstanced,
            FragmentFunction::Circular,
        )
        .with_vertex_input(VertexInput::ColorUv)
        .build(device);

        self.pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("AOS Zero-Copy Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GpuVertex::layout(), particle_layout()],
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

    // Since we are taking data straight from the source storage reference, 
    // update accepts the raw slice directly from your storage wrapper.
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {

        let aspect_ratio = config.width as f32 / config.height as f32;

        let screen_metrics = ScreenUniforms {
            aspect_ratio,
            screen_width: config.width as f32,
            screen_height: config.height as f32,
            _padding: 0.0,
        };

        if let Some(buf) = &self.aspect_buffer {
            queue.write_buffer(buf, 0, bytemuck::bytes_of(&screen_metrics));
        }


        // If there's no data or the simulation array is empty, clear out and escape
        if self.raw_data_ptr.is_null() || self.raw_data_bytes == 0 {
            return;
        }

        let required_size = self.raw_data_bytes as u64;

        // Reconstruct the slice safely using the properties we cached during the sync step
        let raw_byte_slice = unsafe {
            std::slice::from_raw_parts(self.raw_data_ptr, self.raw_data_bytes)
        };

        if let Some(buf) = &self.instance_buffer {
            if buf.size() >= required_size {
                // Instantly upload the raw simulation array in one direct memcpy
                queue.write_buffer(buf, 0, raw_byte_slice);
                return;
            }
        }

       

        // Reallocate the vertex instance buffer if the particle array grew
        self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("AOS Raw Simulation Direct Buffer"),
            contents: raw_byte_slice,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }));
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 || self.unit_quad_buffer.is_none() || self.instance_buffer.is_none() || self.pipeline.is_none() {
            return;
        }
        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        
        // NEW: Attach your aspect ratio buffer to slot group index 0
        pass.set_bind_group(0, self.aspect_bind_group.as_ref().unwrap(), &[]);
        
        pass.set_vertex_buffer(0, self.unit_quad_buffer.as_ref().unwrap().slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..6, 0..self.instance_count); 
    }
}

// /// Per-particle instance data sent to GPU
// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct ParticleInstance {
//     pub position: [f32; 3],  // Simulation space position (note: field name must match WGSL)
//     pub radius_x: f32,       // Particle X radius in NDC units (compensates for aspect)
//     pub radius_y: f32,       // Particle Y radius in NDC units (compensates for aspect)
//     pub color: [f32; 4],     // RGBA color
//     pub _padding: f32,       // Align to 16 bytes
// }
// impl ParticleInstance {
//     fn layout() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 // position
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 4,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 // radius_x
//                 wgpu::VertexAttribute {
//                     offset: 12,
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32,
//                 },
//                 // radius_y
//                 wgpu::VertexAttribute {
//                     offset: 16,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32,
//                 },
//                 // color
//                 wgpu::VertexAttribute {
//                     offset: 20,
//                     shader_location: 7,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//             ],
//         }
//     }
// }



// /// A renderer for ANY AoS simulation data using instanced rendering.
// ///
// /// Renders each particle as a quad. Uses one static unit quad vertex buffer,
// /// and a per-particle instance buffer containing position, radius, color.
// /// GPU scales and positions each instance.
// pub struct AosSimulationRenderer<I> {
//     to_instance: Box<dyn Fn(&I) -> ParticleInstance>,
//     /// Persistent CPU-side staging buffer for instance data
//     staged_instances: Vec<ParticleInstance>,
//     pipeline: Option<wgpu::RenderPipeline>,
//     unit_quad_buffer: Option<wgpu::Buffer>,  // Static 6-vertex unit quad
//     instance_buffer: Option<wgpu::Buffer>,
//     instance_count: u32,
//     transform: Transform,
// }

// impl<I> AosSimulationRenderer<I> {
//     /// Create a new instanced particle renderer.
//     ///
//     /// - `to_instance`: maps an item to instance data (position, radius, color)
//     pub fn new(
//         to_instance: impl Fn(&I) -> ParticleInstance + 'static,
//     ) -> Self {
//         Self {
//             to_instance: Box::new(to_instance),
//             staged_instances: Vec::new(),
//             pipeline: None,
//             unit_quad_buffer: None,
//             instance_buffer: None,
//             instance_count: 0,
//             transform: Transform::identity(),
//         }
//     }

//     /// Update the coordinate transform from simulation space to NDC.
//     pub fn set_transform(&mut self, transform: Transform) {
//         self.transform = transform;
//     }
// }

// impl<I: 'static> Renderer for AosSimulationRenderer<I> {
//     type Data = ();

//     fn prepare(
//         &mut self,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.pipeline.is_some() {
//             return;
//         }

//         // Create static unit quad vertices (from -1 to 1)
//         let unit_quad_vertices = [
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0], color: [1.0; 4] },
//         ];

//         self.unit_quad_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Unit Quad Vertex Buffer"),
//             contents: bytemuck::cast_slice(&unit_quad_vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         }));

//         // Instanced particle shader
//         let shader = ShaderBuilder::new(
//             VertexOutput::ColorUv,
//             VertexFunction::ParticleAosInstanced,
//             FragmentFunction::Circular,
//         )
//         .with_vertex_input(VertexInput::ColorUv)
//         .label("AOS Instanced Simulation Shader")
//         .build(device);

//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("AOS Instanced Simulation Pipeline Layout"),
//             bind_group_layouts: &[],
//             immediate_size: 0,
//         });

//         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("AOS Instanced Simulation Pipeline"),
//             layout: Some(&pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: Some("vs_main"),
//                 buffers: &[GpuVertex::layout(), ParticleInstance::layout()],
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: &shader,
//                 entry_point: Some("fs_main"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: config.format,
//                     blend: Some(wgpu::BlendState::ALPHA_BLENDING),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: wgpu::PrimitiveTopology::TriangleList,
//                 ..Default::default()
//             },
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//         self.pipeline = Some(pipeline);
//     }

//     fn set_data(&mut self, _data: Self::Data) {}

//     fn update(
//         &mut self, 
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.staged_instances.is_empty() {
//             self.instance_count = 0;
//             return;
//         }

//         self.instance_count = self.staged_instances.len() as u32;
//         let required_size = (self.staged_instances.len() * std::mem::size_of::<ParticleInstance>()) as u64;
//         if let Some(buf) = &self.instance_buffer {
//             if buf.size() >= required_size {
//                 queue.write_buffer(buf, 0, bytemuck::cast_slice(&self.staged_instances));
//                 return;
//             }
//         }
//         self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("AOS Simulation Instance Buffer"),
//             contents: bytemuck::cast_slice(&self.staged_instances),
//             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//         }));
//     }

//     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
//         if self.instance_count == 0 || self.unit_quad_buffer.is_none() || self.instance_buffer.is_none() || self.pipeline.is_none() {
//             return;
//         }
//         pass.set_pipeline(self.pipeline.as_ref().unwrap());
//         pass.set_vertex_buffer(0, self.unit_quad_buffer.as_ref().unwrap().slice(..));
//         pass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..));
//         pass.draw(0..6, 0..self.instance_count);  // 6 vertices per quad, N instances
//     }
// }

// /// Implement `SimulationRenderer` generically for any `AosCpuStorage` whose item matches `I`.
// impl<I: Clone + 'static, S: AosCpuStorage<Item = I>> SimulationRenderer<S> for AosSimulationRenderer<I> {
//     /// Collect instance data from storage. Transform positions from simulation space to NDC on CPU.
//     fn sync(&mut self, storage: &S, _config: &wgpu::SurfaceConfiguration) {
//         let items = storage.as_slice();

//         self.staged_instances.clear();
//         self.staged_instances.reserve(items.len());
//         for item in items {
//             let mut instance = (self.to_instance)(item);
            
//             // Transform position from simulation space to NDC
//             let [sim_x, sim_y, sim_z] = [instance.position[0] as f64, instance.position[1] as f64, instance.position[2] as f64];
//             let [ndc_x, ndc_y, ndc_z] = self.transform.sim_to_ndc(sim_x, sim_y, sim_z);
//             instance.position = [ndc_x as f32, ndc_y as f32, ndc_z as f32];
            
//             // Transform radius from simulation units to NDC units separately for X and Y
//             // This ensures particles stay circular regardless of window aspect ratio
//             instance.radius_x = (instance.radius_x as f64 * self.transform.scale[0].abs()) as f32;
//             instance.radius_y = (instance.radius_y as f64 * self.transform.scale[1].abs()) as f32;
            
//             self.staged_instances.push(instance);
//         }
//     }

//     fn set_transform(&mut self, transform: Transform) {
//         self.transform = transform;
//     }
// }

 

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct ParticleInstance {
//     pub position: [f32; 3],  // Simulation space position (note: field name must match WGSL)
//     pub radius: f32,       // Particle radius in NDC units (compensates for aspect) 
//     pub color: [f32; 4],     // RGBA color 
// }

// impl ParticleInstance {
//     fn layout() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 // position
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 4,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 // radius
//                 wgpu::VertexAttribute {
//                     offset: 12,
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32,
//                 },
//                 // color
//                 wgpu::VertexAttribute {
//                     offset: 16,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//             ],
//         }
//     }
// }



// /// A renderer for ANY AoS simulation data using instanced rendering.
// ///
// /// Renders each particle as a quad. Uses one static unit quad vertex buffer,
// /// and a per-particle instance buffer containing position, radius, color.
// /// GPU scales and positions each instance.
// pub struct AosSimulationRenderer<I> {
//     to_instance: Box<dyn Fn(&I) -> ParticleInstance>,
//     /// Persistent CPU-side staging buffer for instance data
//     staged_instances: Vec<ParticleInstance>,
//     pipeline: Option<wgpu::RenderPipeline>,
//     unit_quad_buffer: Option<wgpu::Buffer>,  // Static 6-vertex unit quad
//     instance_buffer: Option<wgpu::Buffer>,
//     instance_count: u32,
//     transform: Transform,
// }

// impl<I> AosSimulationRenderer<I> {
//     /// Create a new instanced particle renderer.
//     ///
//     /// - `to_instance`: maps an item to instance data (position, radius, color)
//     pub fn new(
//         to_instance: impl Fn(&I) -> ParticleInstance + 'static,
//     ) -> Self {
//         Self {
//             to_instance: Box::new(to_instance),
//             staged_instances: Vec::new(),
//             pipeline: None,
//             unit_quad_buffer: None,
//             instance_buffer: None,
//             instance_count: 0,
//             transform: Transform::identity(),
//         }
//     }

//     /// Update the coordinate transform from simulation space to NDC.
//     pub fn set_transform(&mut self, transform: Transform) {
//         self.transform = transform;
//     }
// }


// impl<I: 'static> Renderer for AosSimulationRenderer<I> {
//     type Data = ();

//     fn prepare(
//         &mut self,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.pipeline.is_some() {
//             return;
//         }

//         // FIX: Transformed UV space to center (0.0, 0.0) at the absolute middle of the quad.
//         // This is necessary for fragment mathematical circle functions (length(uv) < 1.0).
//         let unit_quad_vertices = [
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0,  1.0], color: [1.0; 4] },
//         ];

//         self.unit_quad_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Unit Quad Vertex Buffer"),
//             contents: bytemuck::cast_slice(&unit_quad_vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         }));

//         let shader = ShaderBuilder::new(
//             VertexOutput::ColorUv,
//             VertexFunction::ParticleAosInstanced,
//             FragmentFunction::Circular,
//         )
//         .with_vertex_input(VertexInput::ColorUv)
//         .label("AOS Instanced Simulation Shader")
//         .build(device);

//         // NOTE: If you notice rendering anomalies or aspect ratio stretches, 
//         // make sure your ShaderBuilder binds window projection metrics via this layout!
//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("AOS Instanced Simulation Pipeline Layout"),
//             bind_group_layouts: &[],
//             immediate_size: 0,
//         });

//         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("AOS Instanced Simulation Pipeline"),
//             layout: Some(&pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: Some("vs_main"),
//                 buffers: &[GpuVertex::layout(), ParticleInstance::layout()],
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: &shader,
//                 entry_point: Some("fs_main"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: config.format,
//                     blend: Some(wgpu::BlendState::ALPHA_BLENDING),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: wgpu::PrimitiveTopology::TriangleList,
//                 ..Default::default()
//             },
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//         self.pipeline = Some(pipeline);
//     }

//     fn set_data(&mut self, _data: Self::Data) {}

//     fn update(
//         &mut self, 
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.staged_instances.is_empty() {
//             self.instance_count = 0;
//             return;
//         }

//         self.instance_count = self.staged_instances.len() as u32;
//         let required_size = (self.staged_instances.len() * std::mem::size_of::<ParticleInstance>()) as u64;
        
//         // Verify buffer size is large enough. Reallocate only if we outgrow it.
//         if let Some(buf) = &self.instance_buffer {
//             if buf.size() >= required_size {
//                 queue.write_buffer(buf, 0, bytemuck::cast_slice(&self.staged_instances));
//                 return;
//             }
//         }
        
//         // Reallocate buffer on demand
//         self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("AOS Simulation Instance Buffer"),
//             contents: bytemuck::cast_slice(&self.staged_instances),
//             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//         }));
//     }

//     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
//         if self.instance_count == 0 || self.unit_quad_buffer.is_none() || self.instance_buffer.is_none() || self.pipeline.is_none() {
//             return;
//         }
//         pass.set_pipeline(self.pipeline.as_ref().unwrap());
//         pass.set_vertex_buffer(0, self.unit_quad_buffer.as_ref().unwrap().slice(..));
//         pass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..));
//         pass.draw(0..6, 0..self.instance_count); 
//     }
// }


// /// Implement `SimulationRenderer` generically for any `AosCpuStorage` whose item matches `I`. 
// impl<I: Clone + 'static, S: AosCpuStorage<Item = I>> SimulationRenderer<S> for AosSimulationRenderer<I> {
//     /// Collect instance data from storage. Transform positions from simulation space to NDC on CPU.
//     fn sync(&mut self, storage: &S, _config: &wgpu::SurfaceConfiguration) {
//         let items = storage.as_slice();

//         self.staged_instances.clear();
//         self.staged_instances.reserve(items.len());
        
//         // Performance optimization: Cache scale scalar outside the loop
//         let scale_factor = self.transform.scale[0].abs() as f32;

//         for item in items {
//             let mut instance = (self.to_instance)(item);
            
//             // Transform position from simulation space to NDC
//             let sim_x = instance.position[0] as f64;
//             let sim_y = instance.position[1] as f64;
//             let sim_z = instance.position[2] as f64;
            
//             let [ndc_x, ndc_y, ndc_z] = self.transform.sim_to_ndc(sim_x, sim_y, sim_z);
//             instance.position = [ndc_x as f32, ndc_y as f32, ndc_z as f32];
            
//             // Pass uniform simulation radius size to the instance.
//             // Shader handles X/Y aspect skewing seamlessly via a Uniform.
//             instance.radius = instance.radius * scale_factor;
            
//             self.staged_instances.push(instance);
//         }
//     }

//     fn set_transform(&mut self, transform: Transform) {
//         self.transform = transform;
//     }
// }

 