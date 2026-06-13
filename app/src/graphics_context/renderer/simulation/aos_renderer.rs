use wgpu::util::DeviceExt;
use crate::graphics_context::{
    renderer::Renderer,
    vertex::GpuVertex,
    shader::{
        ShaderBuilder, 
    },
};

/// A renderer for ANY AoS-stored simulation data.
/// Generic over the item type `I` - works for particles, rigid bodies, fluids, etc.
/// 
/// # Type Parameter
/// - `I`: The simulation item type (e.g., `VerletParticle2d`, `RigidBody3d`)
///        This type comes from `AosCpuStorage::Item` but the renderer doesn't
///        need to know about the storage trait.
pub struct AosSimulationRenderer<I> {
    data: Vec<I>,  // Owns a snapshot of the simulation data
    to_vertex: Box<dyn Fn(&I) -> GpuVertex>,  // Converts simulation item → GPU vertex
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl<I> AosSimulationRenderer<I> {
    /// Create a new renderer with initial data and a conversion function.
    /// 
    /// # Arguments
    /// - `initial_data`: Snapshot of simulation items (extracted via `.as_slice().to_vec()`)
    /// - `to_vertex`: Function that converts a simulation item to a GPU vertex
    /// 
    /// # Example
    /// ```ignore
    /// let renderer = AosSimulationRenderer::new(
    ///     particles,
    ///     |p| GpuVertex {
    ///         position: [p.pos[0] as f32, p.pos[1] as f32, 0.0],
    ///         color: [1.0, 0.5, 0.2, 1.0],
    ///     }
    /// );
    /// ```
    pub fn new(
        initial_data: Vec<I>,
        to_vertex: impl Fn(&I) -> GpuVertex + 'static,
    ) -> Self {
        Self {
            data: initial_data,
            to_vertex: Box::new(to_vertex),
            pipeline: None,
            vertex_buffer: None,
        }
    }
}

impl<I: 'static> Renderer for AosSimulationRenderer<I> {
    type Data = Vec<I>;
    
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipeline.is_some() {
            return;
        }

        let shader = ShaderBuilder::aos_color_particle().build(device);

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
    
    fn update_data(&mut self, data: Self::Data) {
        self.data = data;
    }
    
    fn update(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
    ) {
        if self.data.is_empty() {
            return;
        }

        // Convert simulation items → GPU vertices using the conversion function
        let gpu_vertices: Vec<GpuVertex> = self.data
            .iter()
            .map(|item| (self.to_vertex)(item))  // `I` → `GpuVertex`
            .collect();

        // Upload to GPU
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("AOS Simulation Vertex Buffer"),
            contents: bytemuck::cast_slice(&gpu_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
    }
    
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.data.is_empty() || self.vertex_buffer.is_none() || self.pipeline.is_none() {
            return;
        }

        pass.set_pipeline(self.pipeline.as_ref().unwrap());
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.draw(0..self.data.len() as u32, 0..1);
    }
}


// use wgpu::util::DeviceExt;
// use crate::graphics_context::{
//     renderer::Renderer,
//     vertex::GpuVertex,
//     shader::{
//         ShaderBuilder,
//         vertex::VertexFunction,
//         vertex_input::VertexInput,
//         vertex_output::VertexOutput,
//         fragment::FragmentFunction,
//     },
// };

// /// A renderer for ANY AoS-stored simulation data.
// /// Generic over the item type `I` - works for particles, rigid bodies, fluids, etc.
// pub struct AosSimulationRenderer<I> {
//     data: Vec<I>,  // Owns a snapshot of the simulation data
//     to_vertex: Box<dyn Fn(&I) -> GpuVertex>,  // Converts I -> GPU vertex
//     pipeline: Option<wgpu::RenderPipeline>,
//     vertex_buffer: Option<wgpu::Buffer>,
// }

// impl<I> AosSimulationRenderer<I> {
//     /// Create a new renderer with an initial dataset and a conversion function.
//     pub fn new(
//         initial_data: Vec<I>,
//         to_vertex: impl Fn(&I) -> GpuVertex + 'static,
//     ) -> Self {
//         Self {
//             data: initial_data,
//             to_vertex: Box::new(to_vertex),
//             pipeline: None,
//             vertex_buffer: None,
//         }
//     }
// }

// impl<I: 'static> Renderer for AosSimulationRenderer<I> {
//     type Data = Vec<I>;
    
//     fn prepare(
//         &mut self,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.pipeline.is_some() {
//             return;
//         }

//         // Build shader using modular system
//         let shader = ShaderBuilder::new(
//             VertexOutput::Color,
//             VertexFunction::ParticleAosColor,
//             FragmentFunction::Circular,
//         )
//         .label("AOS Simulation Shader")
//         .with_vertex_input(VertexInput::Color)
//         .build(device);

//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("AOS Simulation Pipeline Layout"),
//             bind_group_layouts: &[],
//             immediate_size: 0,
//         });

//         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("AOS Simulation Pipeline"),
//             layout: Some(&pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: Some("vs_main"),
//                 buffers: &[GpuVertex::layout()],
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
//                 topology: wgpu::PrimitiveTopology::PointList,
//                 ..Default::default()
//             },
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//         self.pipeline = Some(pipeline);
//     }
    
//     fn update_data(&mut self, data: Self::Data) {
//         self.data = data;
//     }
    
//     fn update(
//         &mut self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _config: &wgpu::SurfaceConfiguration,
//     ) {
//         if self.data.is_empty() {
//             return;
//         }

//         // Convert simulation data to GPU vertices using the provided closure
//         let gpu_verts: Vec<GpuVertex> = self.data.iter()
//             .map(|item| (self.to_vertex)(item))
//             .collect();

//         let required_size = (gpu_verts.len() * std::mem::size_of::<GpuVertex>()) as u64;

//         // Reuse buffer if large enough, otherwise create new one
//         if let Some(buffer) = &self.vertex_buffer {
//             if buffer.size() >= required_size {
//                 queue.write_buffer(buffer, 0, bytemuck::cast_slice(&gpu_verts));
//                 return;
//             }
//         }

//         self.vertex_buffer = Some(
//             device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some("AOS Simulation Vertex Buffer"),
//                 contents: bytemuck::cast_slice(&gpu_verts),
//                 usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//             })
//         );
//     }
    
//     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
//         if self.data.is_empty() || self.vertex_buffer.is_none() || self.pipeline.is_none() {
//             return;
//         }

//         pass.set_pipeline(self.pipeline.as_ref().unwrap());
//         pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
//         pass.draw(0..self.data.len() as u32, 0..1);
//     }
// }



// // use wgpu::{Device, Queue, SurfaceConfiguration, util::DeviceExt};
// // use crate::graphics_context::{
// //     renderer::Renderer,
// //     vertex::GpuVertex,
// //     shader::{
// //         ShaderBuilder,
// //         vertex::VertexFunction,
// //         vertex_input::VertexInput,
// //         vertex_output::VertexOutput,
// //         fragment::FragmentFunction,
// //     },
// // };
// // use base::sim::storage::AosCpuStorage;
// // use impls::simulation::particle::particle_2d::VerletParticle2d;

// // /// A concrete renderer that knows how to draw particles from an AoS storage.
// // pub struct ParticleRenderer {
// //     data: Vec<Mesh>,
// //     pipeline: Option<wgpu::RenderPipeline>,
// //     vertex_buffer: Option<wgpu::Buffer>,
// //     particle_count: u32,
// // }

// // impl ParticleRenderer {
// //     pub fn new() -> Self {
// //         Self {
// //             pipeline: None,
// //             vertex_buffer: None,
// //             particle_count: 0,
// //         }
// //     }
// // }

// // // This renderer can draw any data `D` that behaves like AoS particle storage.
// // impl<D> Renderer<D> for ParticleRenderer
// // where
// //     D: AosCpuStorage<Item = VerletParticle2d>,
// // {
// //     fn prepare(
// //         &mut self,
// //         device: &wgpu::Device,
// //         _queue: &wgpu::Queue,
// //         config: &wgpu::SurfaceConfiguration,
// //     ) {
// //         if self.pipeline.is_some() { return; }

// //         // Build the shader using the new modular system.
// //         // We're using:
// //         // - VertexInput::Color (position + color)
// //         // - VertexOutput::Color (clip_position + color)
// //         // - VertexFunction::ParticleAosColor (AOS particle vertex shader)
// //         // - FragmentFunction::Circular (draws circular points)
// //         let shader = ShaderBuilder::new(
// //             VertexOutput::Color,
// //             VertexFunction::ParticleAosColor,
// //             FragmentFunction::Circular,
// //         )
// //         .label("AOS Particle Shader")
// //         .with_vertex_input(VertexInput::Color) // AOS needs the input struct
// //         .build(device);

// //         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
// //             label: Some("Particle Pipeline Layout"),
// //             bind_group_layouts: &[],
// //              immediate_size: 0,
// //         });

// //         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
// //             label: Some("Particle Pipeline"),
// //             layout: Some(&pipeline_layout),
// //             vertex: wgpu::VertexState {
// //                 module: &shader,
// //                 entry_point: Some("vs_main"),
// //                 buffers: &[GpuVertex::layout()],
// //                 compilation_options: Default::default(),
// //             },
// //             fragment: Some(wgpu::FragmentState {
// //                 module: &shader,
// //                 entry_point: Some("fs_main"),
// //                 targets: &[Some(wgpu::ColorTargetState {
// //                     format: config.format,
// //                     blend: Some(wgpu::BlendState::ALPHA_BLENDING),
// //                     write_mask: wgpu::ColorWrites::ALL,
// //                 })],
// //                 compilation_options: Default::default(),
// //             }),
// //             primitive: wgpu::PrimitiveState {
// //                 topology: wgpu::PrimitiveTopology::PointList,
// //                 ..Default::default()
// //             },
// //             depth_stencil: None,
// //             multisample: wgpu::MultisampleState::default(),
// //             multiview_mask: None,
// //             cache: None,       
// //         });

// //         self.pipeline = Some(pipeline);
// //     }

// //     fn update(&mut self, device: &Device, queue: &Queue, _config: &SurfaceConfiguration, data: &D) {
// //         let storage_slice = data.as_slice();
// //         self.particle_count = storage_slice.len() as u32;
// //         if self.particle_count == 0 { return; }

// //         let gpu_verts: Vec<GpuVertex> = storage_slice.iter().map(|p| GpuVertex {
// //             position: [p.pos[0] as f32, p.pos[1] as f32, 0.0],
// //             normal: [0.0, 0.0, 1.0],
// //             uv: [0.0, 0.0],
// //             color: [1.0, 0.5, 0.2, 1.0],
// //         }).collect();

// //         // Create or update the vertex buffer
// //         if let Some(buffer) = &self.vertex_buffer {
// //             if buffer.size() >= gpu_verts.len() as u64 * std::mem::size_of::<GpuVertex>() as u64 {
// //                 queue.write_buffer(buffer, 0, bytemuck::cast_slice(&gpu_verts));
// //                 return;
// //             }
// //         }

// //         self.vertex_buffer = Some(
// //             device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// //                 label: Some("Particle Vertex Buffer"),
// //                 contents: bytemuck::cast_slice(&gpu_verts),
// //                 usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
// //             })
// //         );
// //     }

// //     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
// //         if self.particle_count == 0 || self.vertex_buffer.is_none() {
// //             return;
// //         }

// //         pass.set_pipeline(self.pipeline.as_ref().unwrap());
// //         pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
// //         pass.draw(0..self.particle_count, 0..1);
// //     }
// // }