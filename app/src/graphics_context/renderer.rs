pub mod mesh_renderer;
pub mod particle_renderer;
pub mod text_renderer;

use wgpu::{Device, Queue, SurfaceConfiguration};


/// A trait for a self-contained renderer that owns its data and knows how to draw it.
pub trait Renderer {
    /// The type of data this renderer manages (e.g., Vec<Mesh>, TextParams).
    type Data;

    /// Prepare pipelines, shaders, and other long-lived GPU resources.
    fn prepare(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

    /// Update the renderer's internal data.
    //fn update_data(&mut self, data: Self::Data);

    /// Update GPU buffers with the latest data from its internal state.
    /// This is called every frame before `draw`.
    fn update(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

    /// Issue draw calls for the current frame.
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}


// /// A trait for a dedicated renderer that knows how to draw a specific type of data.
// /// This is the "sub-renderer" that lives inside a `RenderPass`.
// /// It is generic over the data source `D` that it reads from.
// pub trait Renderer<D> {
//     /// Prepare pipelines, shaders, and other long-lived GPU resources.
//     fn prepare(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

//     /// Update GPU buffers with the latest data from the source `D`.
//     /// This is called every frame before `draw`.
//     fn update(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration, data: &D);

//     /// Issue draw calls for the current frame.
//     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
// }