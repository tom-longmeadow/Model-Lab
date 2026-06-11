pub mod simulation_pass;

use super::renderer::Renderer;


/// The main trait for any object that can act as a stage in the rendering pipeline.
pub trait Pass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    );
     fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration);
    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>);
}


/// A generic render pass that holds a data source `D` and a renderer for that data.
pub struct RenderPass<D> {
    pub data: D,
    renderer: Box<dyn Renderer<D>>,
}

impl<D> RenderPass<D> {
    pub fn new(data: D, renderer: impl Renderer<D> + 'static) -> Self {
        Self {
            data,
            renderer: Box::new(renderer),
        }
    }
}

impl<D> Pass for RenderPass<D> {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.renderer.prepare(device, _queue, config);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.renderer.update(device, queue, config, &self.data);
    }

    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}

