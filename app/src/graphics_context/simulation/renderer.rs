use base::sim::storage::Storage;

use crate::graphics_context::renderer::Renderer;

 


/// A [`Renderer`] that knows how to extract its own render data from a simulation storage `S`.
///
/// Implement this on your renderer to make it compatible with [`SimulationPass`].
/// The storage layout (AoS, SoA, GPU-native, etc.) is entirely the renderer's concern —
/// `SimulationPass` never touches the storage directly.
///
/// # Examples
/// - `AosSimulationRenderer<I>` implements `SimulationRenderer<S>` for any `AosCpuStorage<Item = I>`
/// - A future `SoaSimulationRenderer` would implement it by reading per-field column slices
/// - A GPU-native renderer could implement it by recording a buffer copy command
pub trait SimulationRenderer<S: Storage>: Renderer {
    fn sync(&mut self, storage: &S, config: &wgpu::SurfaceConfiguration);
}
 