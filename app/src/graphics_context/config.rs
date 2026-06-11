
pub struct RendererConfig {
    pub clear_color:   wgpu::Color,
    pub present_mode:  wgpu::PresentMode,
    pub depth_enabled: bool,
    pub sample_count:  u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            clear_color:   wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            present_mode:  wgpu::PresentMode::Fifo,
            depth_enabled: true,
            sample_count:  1,
        }
    }
}