#[derive(Clone, Copy)]
pub enum VertexOutput {
    Color,
    Lit,
    RadiusColor,
    RadiusLit,
}

impl VertexOutput {
    pub fn source(&self) -> &'static str {
        match self {
            VertexOutput::Color => include_str!("color.wgsl"),
            VertexOutput::Lit => include_str!("lit.wgsl"),
            VertexOutput::RadiusColor => include_str!("radius_color.wgsl"),
            VertexOutput::RadiusLit => include_str!("radius_lit.wgsl"),
        }
    }
}