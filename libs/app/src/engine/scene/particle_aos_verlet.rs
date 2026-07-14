use std::sync::{Arc, Mutex};
use base::{  math::{Bounds, DVec2}, sim::{simulation::Simulation, solver::constraint::Insets, storage::CpuStorage}, ui::layout::color::Color };
use impls::simulation::verlet_2d::{aos_vec_storage::AosVecStorage, lifecycle::stream::StreamLifecycle, particle::Particle, solver::gravity::VerletGravitySolver};

use crate::{
    engine::{input::InputState, scene::Scene},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState},
        simulation::{aos::AosSimulationRenderer, pass::{SimulationPass}},
    }, 
};

pub struct ParticleAosVerletScene { 
    hud_state: Arc<Mutex<HudState>>,
}

impl ParticleAosVerletScene {
    pub fn new() -> Self {
        Self { 
            hud_state: Arc::new(Mutex::new(HudState::default())),
        }
    }
}

impl Scene for ParticleAosVerletScene {
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {

        if renderer.pass_count() > 0 {
            return;
        }

        let hz: f64 = 60.0;
        let substep_count: u64 = 4;
        let collision_iterations: u64 = 2;
        let insets: Insets = Insets::symmetrical(10.0, 30.0);
        let gravity: f64 = 1400.0;  
        let lifecycle_start_tick: u64 = 50;
        let lifecycle_ticks_per_spawn: u64 = 2;  
        let max_particles: usize = 250;
        let particle_initial_velocity: DVec2 = DVec2 { x: 6.0, y: -3.0 };
        let particle_radius: f64 = 10.0;  
        let particle_colors: &'static [Color] = &Color::RAINBOW;
        
        
        let sim = Simulation::new(
            hz,
            <AosVecStorage as CpuStorage>::new(max_particles),
            VerletGravitySolver::new(substep_count, collision_iterations, gravity, insets),
            StreamLifecycle::new(lifecycle_start_tick, lifecycle_ticks_per_spawn, max_particles, 
                                            particle_initial_velocity, particle_radius, particle_colors),
            Bounds::default(), // in the gravity solver, the bounds is auto calculated each frame.
        );   
         

        let particle_renderer = AosSimulationRenderer::<Particle>::new(); 
        renderer.add_pass(SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone()));
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
        
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        // if let Some(ui) = &mut self.ui {
        //     let changes = ui.drain_changes();
        //     if !changes.is_empty() {
        //         println!("Properties changed: {:?}", changes);
        //     }
        // }
    }

    
}
