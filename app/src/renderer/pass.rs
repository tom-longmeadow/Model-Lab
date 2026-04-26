pub mod text;

pub trait RenderPass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    );
    
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>);
}
