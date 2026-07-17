 
use wgpu::util::DeviceExt;
use base::{math::{DVec2}, sim::{solver::particle::verlet_particle::VerletParticle, storage::AosCpuStorage}};
use crate::graphics_context::{
     renderer::Renderer, shader::{
        ShaderBuilder,
        fragment::FragmentFunction,
        vertex::VertexFunction,
        vertex_input::VertexInput,
        vertex_output::VertexOutput,
    }, simulation::renderer::SimulationRenderer, state::{quad_state::QuadState} 
};

pub type MyVerletParticle = VerletParticle<DVec2>; 

fn particle_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: MyVerletParticle::STRIDE as wgpu::BufferAddress,
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
    quad_state: Option<QuadState>, // Holds ONLY the screen metrics uniform assets now
    instance_buffer: Option<wgpu::Buffer>,
    instance_count: u32, 

    // Track pointer states safely between sync and update passes
    raw_data_ptr: *const u8,
    raw_data_bytes: usize,
    
    _marker: std::marker::PhantomData<I>,
}

impl<I> AosSimulationRenderer<I> {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            quad_state: None,
            instance_buffer: None,
            instance_count: 0, 
            raw_data_ptr: std::ptr::null(),
            raw_data_bytes: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S: AosCpuStorage<Item = MyVerletParticle>> SimulationRenderer<S> for AosSimulationRenderer<MyVerletParticle> {
    fn sync(&mut self, storage: &S, _config: &wgpu::SurfaceConfiguration) {
        let items = storage.as_slice();
        self.instance_count = items.len() as u32;

        if items.is_empty() {
            self.raw_data_ptr = std::ptr::null();
            self.raw_data_bytes = 0;
            return;
        } 
        self.raw_data_ptr = items.as_ptr() as *const u8;
        self.raw_data_bytes = items.len() * (MyVerletParticle::STRIDE as usize);
    }
}

impl<I: 'static> Renderer for AosSimulationRenderer<I> {
    type Data = ();

    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        if self.pipeline.is_some() { return; }

        // 1. Compile the shader code first
        let shader = ShaderBuilder::new(
            VertexOutput::ColorUv,
            VertexFunction::ParticleAosInstanced,
            FragmentFunction::Circular,
        )
        .with_vertex_input(VertexInput::ColorUv)
        .build(device);

        // 2. Build the pipeline with NO explicit layout (wgpu will auto-reflect the WGSL)
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("AOS Zero-Copy Pipeline"),
            layout: None, // Instructs wgpu to auto-generate the pipeline layout
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // FIX: GpuVertex::layout() removed. particle_layout() slides down to index 0.
                buffers: &[particle_layout()], 
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

        // 3. Extract the automatically generated layout from Bind Group Index 0
        let inferred_layout = pipeline.get_bind_group_layout(0);

        // 4. Pass the inferred layout into QuadState so it can build its Bind Group
        let quad_state = QuadState::new(
            device, 
            &inferred_layout, 
            config.width as f32, 
            config.height as f32
        );

        // 5. Store the clean assets in your renderer state
        self.pipeline = Some(pipeline);
        self.quad_state = Some(quad_state);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        // 1. Let the quad state internally update and upload screen uniforms
        if let Some(ref mut quad_state) = self.quad_state {
            quad_state.resize(queue, config.width as f32, config.height as f32);
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
        return;
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        // Extract references and safely escape if any core assets are missing
        let pipeline = match &self.pipeline { Some(p) => p, None => return };
        let quad_state = match &self.quad_state { Some(q) => q, None => return };
        let instance_buffer = match &self.instance_buffer { Some(b) => b, None => return };
        
        if self.instance_count == 0 {
            return;
        }

        // 1. Bind the pipeline setup
        pass.set_pipeline(pipeline);
        
        // 2. Attach the screen metrics uniform bind group from QuadState
        pass.set_bind_group(0, &quad_state.screen_metrics.bind_group, &[]);

        // 3. Set up the particle instance data layout onto Vertex Slot 0
        // FIX: Removed the static quad vertex buffer mapping. Bound your instance data straight into slot 0.
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        
        // FIX: Removed index buffer attachment step completely.

        // 4. Run unindexed draw command
        // FIX: Replaced draw_indexed with draw(0..6, ...). The WGSL shader generates vertices and maps indices manually.
        pass.draw(0..6, 0..self.instance_count); 
    }
}