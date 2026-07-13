use std::sync::{Arc, Mutex};
use base::{math::Bounds, sim::{metrics::SimMetrics, simulation::Simulate}};
use crate::graphics_context::{pass::{Pass, hud::HudState}, simulation::{renderer::SimulationRenderer, Transform}};

/// Strategy for handling window resize events in relation to simulation bounds.
//#[derive(Debug, Clone, Copy)]
// pub enum ResizeStrategy {
//     /// Simulation units are pixels. Window size determines world size.
//     /// Particle radius in pixels stays constant visual size regardless of window size.
//     /// Like a marble in a box - resize window = resize box, marble stays same size.
//     Dynamic,
// }

/// A render pass that owns a simulation and a renderer, driving both each frame.
///
/// `SimulationPass` is layout-agnostic — it does not know or care whether the
/// simulation uses AoS, SoA, or a GPU-native layout. That is entirely the
/// renderer's responsibility, expressed through `SimulationRenderer::sync`.
///
/// On each frame `update()`:
///   1. Advances the simulation by `dt` seconds
///   2. Calls `renderer.sync(storage)` — renderer extracts what it needs
///   3. Calls `renderer.update(...)` — renderer uploads to GPU
pub struct SimulationPass<S, R>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
{
    simulation: S,
    renderer: R, 
    hud: Option<Arc<Mutex<HudState>>>,
    //strategy: ResizeStrategy,
    //transform: Transform,
    sim_bounds: Bounds, 
}

impl<S, R> SimulationPass<S, R>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
{
    pub fn new(simulation: S, renderer: R, sim_bounds: Bounds) -> Self {
        Self { 
            simulation, 
            renderer,  
            hud: None,
            //strategy: ResizeStrategy::Dynamic,
            //transform: Transform::identity(),
            sim_bounds, 
        }
    }

    /// Attach a shared HudState so the pass automatically writes sim metrics each frame.
    pub fn with_hud(mut self, hud: Arc<Mutex<HudState>>) -> Self {
        self.hud = Some(hud);
        self
    }

    // /// Set the resize strategy for this simulation pass.
    // pub fn with_strategy(mut self, strategy: ResizeStrategy) -> Self {
    //     self.strategy = strategy;
    //     self
    // }

    // /// Calculate transform based on current strategy and surface config.
    // fn calculate_transform(&self, _config: &wgpu::SurfaceConfiguration) -> Transform {
    //     match self.strategy {
    //         ResizeStrategy::Dynamic => {
    //             // Simulation units are pixels. Map sim bounds to full NDC range.
    //             // Since bounds match window dimensions, this gives 1 sim unit = 1 pixel.
    //             Transform::from_bounds(
    //                 self.sim_bounds.min.x, self.sim_bounds.max.x,
    //                 self.sim_bounds.min.y, self.sim_bounds.max.y,
    //                 -1.0, 1.0,
    //                 -1.0, 1.0,
    //             )
    //         }
    //     }
    // }
}

impl<S, R> Pass for SimulationPass<S, R>
where
    S: Simulate + 'static,
    R: SimulationRenderer<S::Storage> + 'static,
{
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        // For Dynamic strategy, bounds = window dimensions in pixels
        // Bottom-left stays at (0,0), top-right expands to (width, height)
        let new_bounds = Bounds::new_2d((0.0, 0.0),(config.width as f64, config.height as f64));
            
        
        self.simulation.set_bounds(new_bounds);
        self.sim_bounds = new_bounds;
        
        self.renderer.prepare(device, queue, config);
        
        // // Calculate transform after potentially updating sim_bounds
        // self.transform = Self::calculate_transform(self, config);
        // self.renderer.set_transform(self.transform);
    }

    fn update(&mut self, frame_time: f64, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.simulation.simulate(frame_time);
 
        if let Some(hud) = &self.hud {
            if let Ok(mut s) = hud.try_lock() {
                let m = self.simulation.metrics();
                
                s.set("FPS",format!("{:.0}", if frame_time > 0.0 { 1.0 / frame_time } else { 0.0 }));
                s.set("",    "");
                s.set("Particles", format!("{:.0}", m.storage_size));
                s.set("hz", format!("{:.0}",       m.hz));
                s.set("Max step",        format!("{:.3} ms",    1.0 / m.hz * 1000.0));
                s.set("Step time",       format!("{:.3} ms",    m.step_time_ms));
                // s.set("Steps/f",   format!("{}",          m.steps_per_frame)); 
                // s.set("Substep",   format!("{:.3} ms",    m.substep_time_ms));
                s.set("Accum",     format!("{:.2} ms",    m.accumulator_ms));
                s.set("Ticks",     format!("{}",          m.total_ticks));
            }
        }

        self.renderer.sync(self.simulation.storage(), config);
        self.renderer.update(device, queue, config);
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}
 