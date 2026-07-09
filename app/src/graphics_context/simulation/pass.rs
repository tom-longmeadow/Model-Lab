use std::sync::{Arc, Mutex};
use base::sim::simulation::Simulate;
use crate::graphics_context::{pass::{Pass, hud::HudState}, simulation::renderer::SimulationRenderer};

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
    dt: f64,
    hud: Option<Arc<Mutex<HudState>>>,
}

impl<S, R> SimulationPass<S, R>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
{
    pub fn new(simulation: S, renderer: R, dt: f64) -> Self {
        Self { simulation, renderer, dt, hud: None }
    }

    /// Attach a shared HudState so the pass automatically writes sim metrics each frame.
    pub fn with_hud(mut self, hud: Arc<Mutex<HudState>>) -> Self {
        self.hud = Some(hud);
        self
    }
}

impl<S, R> Pass for SimulationPass<S, R>
where
    S: Simulate + 'static,
    R: SimulationRenderer<S::Storage> + 'static,
{
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.renderer.prepare(device, queue, config);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.simulation.simulate(self.dt);

        if let Some(hud) = &self.hud {
            if let Ok(mut s) = hud.try_lock() {
                let m = self.simulation.metrics();
                s.set("Sim hz",    format!("{:.0}",      m.hz));
                s.set("Steps/f",   format!("{}",          m.steps_per_frame));
                s.set("Step",      format!("{:.3} ms",    m.step_time_ms));
                s.set("Substep",   format!("{:.3} ms",    m.substep_time_ms));
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
 