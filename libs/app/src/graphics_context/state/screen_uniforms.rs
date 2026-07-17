use crate::graphics_context::state::uniform_layout::{UniformLayout};

 

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniforms {
    pub aspect_ratio: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub _padding: f32,
}

impl UniformLayout for ScreenUniforms {
    const LABEL: &'static str = "Screen Metrics Uniform";
}

impl ScreenUniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            aspect_ratio: width / height,
            screen_width: width,
            screen_height: height,
            _padding: 0.0,
        }
    }
}

 