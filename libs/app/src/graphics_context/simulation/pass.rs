use std::sync::{Arc, Mutex};
use base::{aabb::AABB, math::Vector, sim::{simulation::Simulate, solver::particle::{ environment::ParticleEnvironment, flags::CollisionFlags}}};
use crate::graphics_context::{pass::{Pass, hud::HudState}, simulation::{renderer::SimulationRenderer}};
use std::marker::PhantomData;


pub struct SimulationPass<S, R, F>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
    F: CollisionFlags,
{
    simulation: S,
    renderer: R, 
    hud: Option<Arc<Mutex<HudState>>>, 
    _flags: PhantomData<F>, // Zero runtime footprint marker
}

impl<S, R, F> SimulationPass<S, R, F>
where
    S: Simulate,
    R: SimulationRenderer<S::Storage>,
    F: CollisionFlags,
{
    pub fn new(simulation: S, renderer: R) -> Self {
        Self { 
            simulation, 
            renderer,  
            hud: None, 
            _flags: PhantomData,
        }
    }

    pub fn with_hud(mut self, hud: Arc<Mutex<HudState>>) -> Self {
        self.hud = Some(hud);
        self
    } 
}

 
impl<S, R, V, F> Pass for SimulationPass<S, R, F>
where  
    S: Simulate<Environment = ParticleEnvironment<V, F>> + 'static,   
    V: Vector + From<(f64, f64)> + 'static,   
    F: CollisionFlags + 'static,
    R: SimulationRenderer<S::Storage> + 'static,
{
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        let max_pos = V::from((config.width as f64, config.height as f64));
        let min_pos = V::ZERO;  
        let new_bounds = AABB::new(min_pos, max_pos); 

        // 🚀 DIRECT FIELD ACCESS FLAWLESSLY:
        // Because S::Environment matches ParticleEnvironment<V, F> precisely, 
        // the compiler lets you read fields directly without any helper methods or traits.
        let env = self.simulation.environment();  
        env.space.bounds = new_bounds;  
        
        self.renderer.prepare(device, queue, config);
    }

    fn update(&mut self, frame_time: f64, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.simulation.simulate(frame_time);
 
        if let Some(hud) = &self.hud {
            if let Ok(mut s) = hud.try_lock() {
                let m = self.simulation.metrics();
                
                s.set("FPS", format!("{:.0}", if frame_time > 0.0 { 1.0 / frame_time } else { 0.0 }));
                s.set("", "");
                s.set("Particles", format!("{:.0}", m.storage_size));
                s.set("hz", format!("{:.0}", m.hz));
                s.set("Max step", format!("{:.3} ms", 1.0 / m.hz * 1000.0));
                s.set("Step time", format!("{:.3} ms", m.step_time_ms));
                s.set("Accum", format!("{:.2} ms", m.accumulator_ms));
                s.set("Ticks", format!("{}", m.total_ticks));
            }
        }

        self.renderer.sync(self.simulation.storage(), config);
        self.renderer.update(device, queue, config);
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}