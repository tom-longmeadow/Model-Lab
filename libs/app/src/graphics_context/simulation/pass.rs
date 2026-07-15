use std::sync::{Arc, Mutex};
use base::{aabb::AABB, math::Vector, sim::{simulation::Simulate, solver::particle::{environment::ParticleEnvironment, 
    verlet_aos_vec_storage::VerletParticleAosVecStorage}}};
use crate::graphics_context::{pass::{Pass, hud::HudState}, simulation::{renderer::SimulationRenderer}};


pub struct SimulationPass<S, R>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
{
    simulation: S,
    renderer: R, 
    hud: Option<Arc<Mutex<HudState>>>, 
}

impl<S, R> SimulationPass<S, R>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
{
    pub fn new(simulation: S, renderer: R) -> Self {
        Self { 
            simulation, 
            renderer,  
            hud: None, 
        }
    }

    /// Attach a shared HudState so the pass automatically writes sim metrics each frame.
    pub fn with_hud(mut self, hud: Arc<Mutex<HudState>>) -> Self {
        self.hud = Some(hud);
        self
    } 
}

impl<S, R, V> Pass for SimulationPass<S, R>
where  
    S: Simulate<
        Storage = VerletParticleAosVecStorage<V>, 
        Environment = ParticleEnvironment<V>
    > + 'static,   
    V: Vector + From<(f64, f64)>,  
    R: SimulationRenderer<VerletParticleAosVecStorage<V>> + 'static,
{
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        let max_pos = V::from((config.width as f64, config.height as f64));
        let min_pos = V::ZERO;  
        let new_bounds = AABB::new(min_pos, max_pos); 

        let env = self.simulation.environment();  
        env.space.bounds = new_bounds;  
        
        self.renderer.prepare(device, queue, config);
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
 