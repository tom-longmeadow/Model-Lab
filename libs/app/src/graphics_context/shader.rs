use crate::graphics_context::shader::{fragment::FragmentFunction, vertex::VertexFunction, vertex_input::VertexInput, vertex_output::VertexOutput};

pub mod fragment;
pub mod vertex;
pub mod vertex_input;
pub mod vertex_output; 

pub struct ShaderBuilder {
    label: Option<&'static str>,
    vertex_input: Option<VertexInput>, // Optional: SOA shaders don't need an input struct
    vertex_output: VertexOutput,
    vertex_function: VertexFunction,
    fragment_function: FragmentFunction,
}

impl ShaderBuilder {
    pub fn new(
        vertex_output: VertexOutput,
        vertex_function: VertexFunction,
        fragment_function: FragmentFunction,
    ) -> Self {
        Self {
            label: None,
            vertex_input: None,
            vertex_output,
            vertex_function,
            fragment_function,
        }
    }

    // /// A shader for simple, colored AOS particles drawn as circles.
    // pub fn aos_color_particle() -> Self {
    //     Self::new(
    //         VertexOutput::ColorUv,
    //         VertexFunction::ParticleAosColor,
    //         FragmentFunction::Circular,
    //     )
    //     .with_vertex_input(VertexInput::ColorUv)
    //     .label("AOS Color Particle")
    // }

    // /// A shader for simple, colored SOA particles drawn as circles.
    // pub fn soa_color_particle() -> Self {
    //     // Note: No .with_vertex_input() call for SOA
    //     Self::new(
    //         VertexOutput::ColorUv,
    //         VertexFunction::ParticleSoaColor,
    //         FragmentFunction::Circular,
    //     )
    //     .label("SOA Color Particle")
    // }

    

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    // AOS shaders need an input struct. SOA shaders define inputs inline in vs_main.
    pub fn with_vertex_input(mut self, input: VertexInput) -> Self {
        self.vertex_input = Some(input);
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::ShaderModule {
        let mut source = String::new();

        // 1. Inject VertexInput struct only if needed (AOS)
        if let Some(input) = self.vertex_input {
            source.push_str(input.source());
            source.push('\n');
        }

        // 2. Inject VertexOutput struct
        source.push_str(self.vertex_output.source());
        source.push('\n');

        // 3. Inject the vertex shader (already contains @vertex fn vs_main)
        source.push_str(self.vertex_function.source());
        source.push('\n');

        // 4. Inject the fragment shader (already contains @fragment fn fs_main)
        source.push_str(self.fragment_function.source());
        source.push('\n');

        // NO GENERATED ENTRY POINTS. The files own their own signatures.

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: self.label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }
}