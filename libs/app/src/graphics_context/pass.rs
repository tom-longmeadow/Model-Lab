pub mod hud;

use super::renderer::Renderer;


/// The main trait for any object that can act as a stage in the rendering pipeline.
pub trait Pass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    );
     fn update(&mut self, frame_time: f64, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration);
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}


/// A generic render pass that holds a data source `D` and a renderer for that data.
pub struct RenderPass<R: Renderer + 'static> {
    pub renderer: R,
}

impl<R: Renderer + 'static> RenderPass<R> {
    /// Creates a new RenderPass by taking ownership of a renderer.
    pub fn new(renderer: R) -> Self {
        Self { renderer }
    }
}

impl<R: Renderer + 'static> Pass for RenderPass<R> {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.renderer.prepare(device, queue, config);
    }

    fn update(&mut self, _frame_time: f64, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.renderer.update(device, queue, config);
    }
    
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}


// impl<D> Pass for RenderPass<D> {
//     fn prepare(
//         &mut self,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         config: &wgpu::SurfaceConfiguration,
//     ) {
//         self.renderer.prepare(device, _queue, config);
//     }

//     fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
//         self.renderer.update(device, queue, config, &self.data);
//     }

//     fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
//         self.renderer.draw(pass);
//     }
// }

