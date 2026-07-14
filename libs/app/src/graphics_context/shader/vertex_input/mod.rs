#[derive(Clone, Copy)]
pub enum VertexInput {
    Color,
    ColorUv,
}

impl VertexInput {
    pub fn source(&self) -> &'static str {
        match self {
            VertexInput::Color   => include_str!("color.wgsl"),
            VertexInput::ColorUv => include_str!("color_uv.wgsl"),
        }
    }
}