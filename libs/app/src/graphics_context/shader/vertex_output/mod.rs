#[derive(Clone, Copy)]
pub enum VertexOutput {
    Color,
    ColorUv,
}

impl VertexOutput {
    pub fn source(&self) -> &'static str {
        match self {
            VertexOutput::Color   => include_str!("color.wgsl"),
            VertexOutput::ColorUv => include_str!("color_uv.wgsl"),
        }
    }
}