use base::sim::{simulation::Simulate, storage::AosCpuStorage};
use crate::graphics_context::{pass::Pass, renderer::Renderer};

/// A render pass that owns a simulation and drives both its advancement and rendering.
///
/// On each frame `update()`:
///   1. Advances the simulation by `dt` seconds
///   2. Clones the current storage slice into the renderer
///   3. Uploads that data to the GPU
pub struct SimulationPass<S, R>
where
    S: Simulate,
    S::Storage: AosCpuStorage,
    R: Renderer<Data = Vec<<S::Storage as AosCpuStorage>::Item>>,
{
    simulation: S,
    renderer: R,
    dt: f64,
}

impl<S, R> SimulationPass<S, R>
where
    S: Simulate,
    S::Storage: AosCpuStorage,
    <S::Storage as AosCpuStorage>::Item: Clone,
    R: Renderer<Data = Vec<<S::Storage as AosCpuStorage>::Item>>,
{
    pub fn new(simulation: S, renderer: R, dt: f64) -> Self {
        Self { simulation, renderer, dt }
    }
}

impl<S, R> Pass for SimulationPass<S, R>
where
    S: Simulate + 'static,
    S::Storage: AosCpuStorage,
    <S::Storage as AosCpuStorage>::Item: Clone + 'static,
    R: Renderer<Data = Vec<<S::Storage as AosCpuStorage>::Item>> + 'static,
{
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.renderer.prepare(device, queue, config);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.simulation.simulate(self.dt);
        let data = self.simulation.storage().as_slice().to_vec();
        self.renderer.update_data(data);
        self.renderer.update(device, queue, config);
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}
 