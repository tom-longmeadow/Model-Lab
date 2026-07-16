use std::{marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;
use base::{
    math::Vector, 
    sim::{
        lifecycle::stream_config::StreamConfig, 
        simulation::Simulation, 
        solver::particle::{
            environment::{GravityModel, ParticleEnvironment}, runtime::RuntimeState, space::GridSpace, tuning::SimulationTuning, verlet_particle::VerletParticle, verlet_soa_gravity_solver::VerletSoaGravitySolver, verlet_soa_stream_lifecycle::SoaStreamLifecycle, verlet_soa_vec_storage::VerletParticleSoaVecStorage 
        }, 
        storage::CpuStorage
    }, 
    ui::layout::color::Color
};
 
use crate::{
    engine::{input::InputState, scene::Scene},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState}, simulation::{pass::SimulationPass, renderer::SimulationRenderer, soa::SoaSimulationRenderer}, 
    }, 
};

pub struct BallBounceParticleSoaVerletScene<V: Vector> { 
    hud_state: Arc<Mutex<HudState>>,
    _marker: PhantomData<V>, 
}

impl<V: Vector> BallBounceParticleSoaVerletScene<V> {
    pub fn new() -> Self {
        Self { 
            hud_state: Arc::new(Mutex::new(HudState::default())),
            _marker: PhantomData,
        }
    }
}

impl<V: Vector + 'static> Scene for BallBounceParticleSoaVerletScene<V> 
where
    V::Scalar: From<f64>, 
    V::Quantized: Hash + Eq, 
    // SWAPPED: Bounds updated to map your SoA Renderer to the SoA Vec Storage container
    SoaSimulationRenderer<VerletParticle<V>>: SimulationRenderer<VerletParticleSoaVecStorage<V>>,
    V: From<(f64, f64)>, 
{
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        let max_particles: usize = 300;
        let hz: f64 = 60.0;
        let substep_count: u64 = 8;
        let collision_iterations: u64 = 4;

        let particle_radius: V::Scalar = 10.0.into();  
        let cell_size = particle_radius * 1.0.into();  
        let gravity_force = V::from_f64_array([0.0, -1600.0]);

        let space = GridSpace::new(cell_size);
        let tuning = SimulationTuning::new(substep_count, collision_iterations, particle_radius);
        let state = RuntimeState::new();
        let gravity = GravityModel::Constant(gravity_force);
        let env = ParticleEnvironment::new(space, tuning, state, gravity);

        let particle_initial_velocity = V::from_f64_array([4.0, -1.0]);
        let particle_relative_location = V::from_f64_array([0.2, 0.97]);
        let lifecycle_start_tick: u64 = 50;
        let lifecycle_ticks_per_spawn: u64 = 3;   
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
        
        // SWAPPED: Wired directly into your SoA Storage, Solver, and Lifecycle Manager
        let sim = Simulation::new(
            hz,
            <VerletParticleSoaVecStorage<V> as CpuStorage>::new(max_particles),
            VerletSoaGravitySolver::<V>::new( max_particles),
            SoaStreamLifecycle::<V>::new(stream_config),
            env,
        );   

        // SWAPPED: Direct instantiation of the parallel array renderer pass
        let particle_renderer = SoaSimulationRenderer::<VerletParticle<V>>::new();   
        let pass = SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone());
        renderer.add_pass(pass);
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        // Update logic remains clean
    }
}