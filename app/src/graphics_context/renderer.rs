pub mod mesh_renderer;
pub mod particle_renderer;
pub mod text_renderer;

use wgpu::{Device, Queue, SurfaceConfiguration};

/// A trait for a dedicated renderer that knows how to draw a specific type of data.
/// This is the "sub-renderer" that lives inside a `RenderPass`.
/// It is generic over the data source `D` that it reads from.
pub trait Renderer<D> {
    /// Prepare pipelines, shaders, and other long-lived GPU resources.
    fn prepare(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration);

    /// Update GPU buffers with the latest data from the source `D`.
    /// This is called every frame before `draw`.
    fn update(&mut self, device: &Device, queue: &Queue, config: &SurfaceConfiguration, data: &D);

    /// Issue draw calls for the current frame.
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}