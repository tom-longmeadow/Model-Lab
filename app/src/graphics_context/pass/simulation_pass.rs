// use crate::graphics_context::{pass::Pass, renderer::Renderer};
// use base::sim::simulation::Simulate;

// /// A specialized render pass for running and drawing a simulation.
// /// It implements the `Pass` trait but adds the crucial step of calling `simulate()`
// /// during the `update` phase.
// pub struct SimulationPass<S>
// where
//     S: Simulate,
// {
//     pub simulation: S,
//     renderer: Box<dyn Renderer<S::Storage>>,
// }

// impl<S> SimulationPass<S>
// where
//     S: Simulate,
// {
//     pub fn new(simulation: S, renderer: impl Renderer<S::Storage> + 'static) -> Self {
//         Self {
//             simulation,
//             renderer: Box::new(renderer),
//         }
//     }
// }

// impl<S> Pass for SimulationPass<S>
// where
//     S: Simulate + 'static, // Add 'static bound for boxing
// {
//     fn prepare(
//         &mut self,
//         device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         config: &wgpu::SurfaceConfiguration,
//     ) {
//         self.renderer.prepare(device, _queue, config);
//     }

//     fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
//         // 1. Advance the simulation state.
//         self.simulation.simulate(1.0 / 60.0); // Assuming 60 FPS for now

//         // 2. Pass the updated storage to the renderer so it can update its GPU buffers.
//         self.renderer.update(device, queue, config, self.simulation.storage());
//     }

//     fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
//         // Delegate the actual draw calls to the specialized renderer.
//         self.renderer.draw(pass);
//     }
// }
 