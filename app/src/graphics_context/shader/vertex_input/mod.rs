#[derive(Clone, Copy)]
pub enum VertexInput {
    Color,
    Lit,
    RadiusColor,
    RadiusLit,
}

impl VertexInput {
    pub fn source(&self) -> &'static str {
        match self {
            VertexInput::Color => include_str!("color.wgsl"),
            VertexInput::Lit => include_str!("lit.wgsl"),
            VertexInput::RadiusColor => include_str!("radius_color.wgsl"),
            VertexInput::RadiusLit => include_str!("radius_lit.wgsl"),
        }
    }
}