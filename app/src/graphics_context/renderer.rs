pub mod mesh; 
pub mod text; 

use wgpu::{Device, Queue, SurfaceConfiguration};


/// A trait for a self-contained renderer that owns its data and knows how to draw it.
pub trait Renderer {
    /// The type of data this renderer manages (e.g., Vec<Mesh>, TextParams).
    type Data;

    /// Prepare pipelines, shaders, and other long-lived GPU resources.
    fn prepare(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

    /// Update the renderer's internal data.
    fn set_data(&mut self, _data: Self::Data) {}

    /// Update GPU buffers with the latest data from its internal state.
    /// Called every frame before `draw`.
    fn update(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

    /// Issue draw calls for the current frame.
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}
