use std::{marker::PhantomData, sync::{Arc, Mutex}};
 use std::hash::Hash;
use base::{aabb::AABB, insets::Insets, math::Vector, sim::{lifecycle::stream_config::StreamConfig, simulation::Simulation, solver::particle::{verlet_particle::VerletParticle}, storage::CpuStorage}, ui::layout::color::Color};
use impls::simulation::particle::{verlet_aos_gravity_solver::VerletAosGravitySolver, verlet_aos_stream_lifecycle::AosStreamLifecycle, verlet_aos_vec_storage::AosVecStorage};

use crate::{
    engine::{input::InputState, scene::Scene},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState},
        simulation::{aos::AosSimulationRenderer, pass::SimulationPass, renderer::SimulationRenderer},
    }, 
};


pub struct ParticleAosVerletScene<V: Vector> { 
    hud_state: Arc<Mutex<HudState>>,
    _marker: PhantomData<V>, // Necessary because V is not used in the struct fields
}

impl<V: Vector> ParticleAosVerletScene<V> {
    pub fn new() -> Self {
        Self { 
            hud_state: Arc::new(Mutex::new(HudState::default())),
            _marker: PhantomData,
        }
    }
}

impl<V: Vector + 'static> Scene for ParticleAosVerletScene<V> 
where
    V::Scalar: From<f64>, 
    V::Quantized: Hash + Eq, 
    AosSimulationRenderer<VerletParticle<V>>: SimulationRenderer<AosVecStorage<V>>,
    V: From<(f64, f64)>, 
{
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        // Configuration constants 
        let hz: f64 = 60.0;
        let substep_count: u64 = 4;
        let collision_iterations: u64 = 2;
        
        let padding: V = V::splat(30.0.into()); 
        let insets = Insets::<V>::symmetrical(padding);
         
        let mut gravity_components = vec![0.0.into(); V::DIM]; 
        if V::DIM > 1 {
            gravity_components[1] = (-1400.0).into(); 
        } 
        let gravity: V = V::from_slice(&gravity_components);
        
        let lifecycle_start_tick: u64 = 50;
        let lifecycle_ticks_per_spawn: u64 = 3;  
        let max_particles: usize = 250;
        
        let mut velocity_components = vec![0.0.into(); V::DIM];
        velocity_components[0] = 6.0.into();   
        if V::DIM > 1 {
            velocity_components[1] = (-3.0).into(); 
        }
        let particle_initial_velocity: V = V::from_slice(&velocity_components);

        let mut relative_location_components = vec![0.0.into(); V::DIM];
        relative_location_components[0] = 0.2.into();   
        if V::DIM > 1 {
            relative_location_components[1] = (0.8).into(); 
        } 
        let particle_relative_location: V = V::from_slice(&relative_location_components);
 
        let particle_radius: V::Scalar = 10.0.into();  
        let particle_colors: &'static [Color] = &Color::RAINBOW;

        let stream_config = StreamConfig::<V>::new(
            lifecycle_start_tick, 
            lifecycle_ticks_per_spawn, 
            max_particles, 
            particle_relative_location,
            particle_initial_velocity, 
            particle_radius, 
            particle_colors
        );
        
        let sim = Simulation::new(
            hz,
            <AosVecStorage<V> as CpuStorage>::new(max_particles),
            VerletAosGravitySolver::<V>::new(substep_count, collision_iterations, gravity, insets, max_particles),
            AosStreamLifecycle::<V>::new(stream_config),
            AABB::<V>::default(), 
        );   

        let particle_renderer = AosSimulationRenderer::<VerletParticle<V>>::new();   
        let pass = SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone());
        renderer.add_pass(pass);
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        // Update logic remains clean
    }
}
 



// pub struct ParticleAosVerletScene { 
//     hud_state: Arc<Mutex<HudState>>,
// }

// impl ParticleAosVerletScene {
//     pub fn new() -> Self {
//         Self { 
//             hud_state: Arc::new(Mutex::new(HudState::default())),
//         }
//     }
// }

// impl Scene for ParticleAosVerletScene {
//     fn build_passes(&mut self, renderer: &mut GraphicsContext) {

//         if renderer.pass_count() > 0 {
//             return;
//         }

//         let hz: f64 = 60.0;
//         let substep_count: u64 = 4;
//         let collision_iterations: u64 = 2;
//         let insets: Insets = Insets::symmetrical(10.0, 30.0);
//         let gravity: f64 = 1400.0;  
//         let lifecycle_start_tick: u64 = 50;
//         let lifecycle_ticks_per_spawn: u64 = 2;  
//         let max_particles: usize = 250;
//         let particle_initial_velocity: DVec2 = DVec2 { x: 6.0, y: -3.0 };
//         let particle_radius: f64 = 10.0;  
//         let particle_colors: &'static [Color] = &Color::RAINBOW;

//         let stream_config = StreamConfig::new(lifecycle_start_tick, lifecycle_ticks_per_spawn, 
//                 max_particles, particle_initial_velocity, particle_radius, particle_colors);
        
        
//         let sim = Simulation::new(
//             hz,
//             <AosVecStorage as CpuStorage>::new(max_particles),
//             VerletAosGravitySolver::new(substep_count, collision_iterations, gravity, insets),
//             AosStreamLifecycle::new(stream_config),
//             Bounds::default(), // in the gravity solver, the bounds is auto calculated each frame.
//         );   
         

//         let particle_renderer = AosSimulationRenderer::<Particle>::new(); 
//         renderer.add_pass(SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone()));
//         renderer.add_pass(HudPass::new(self.hud_state.clone()));

//     }

//     fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
//         // if let Some(ui) = &mut self.ui {
//         //     let changes = ui.drain_changes();
//         //     if !changes.is_empty() {
//         //         println!("Properties changed: {:?}", changes);
//         //     }
//         // }
//     }

    
// }
