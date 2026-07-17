 use crate::graphics_context::{state::{screen_uniforms::ScreenUniforms, uniform_layout::{Uniform}}};

pub struct QuadState {
    pub screen_metrics: Uniform<ScreenUniforms>, 
}

impl QuadState {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, width: f32, height: f32) -> Self { 
        let screen_metrics = Uniform::new(device, layout, ScreenUniforms::new(width, height));

        Self { 
            screen_metrics,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: f32, height: f32) { 
        self.screen_metrics.data = ScreenUniforms::new(width, height); 
        self.screen_metrics.upload(queue);
    }
}