use std::{marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;
use base::{
    math::{FloatScalar, Vector}, 
    sim::{ 
        simulation::Simulation, 
        solver::particle::{
            environment::{GravityModel, ParticleEnvironment}, flags::FluidCollisionFlags, 
            lifecycle::{Stream, fountain::FountainLifecycle}, space::{GridSpace, grid_key::GridKey}, state::State, 
            tuning::SimulationTuning, verlet_particle::VerletParticle, 
            verlet_soa_gravity_solver::VerletSoaGravitySolver, verlet_soa_vec_storage::VerletParticleSoaVecStorage 
        },  
    }, ui::layout::color::Color,  
};

use crate::{
    engine::{input::InputState, scene::{Scene }},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState}, simulation::{pass::SimulationPass, renderer::SimulationRenderer, soa::SoaSimulationRenderer}, 
    }, 
};
 

pub struct WaterFountainParticleSoaVerletScene<V: Vector> { 
    hud_state: Arc<Mutex<HudState>>,
    _marker: PhantomData<V>, 
}

impl<V: Vector + 'static> WaterFountainParticleSoaVerletScene<V> 
where
    V::Scalar: FloatScalar + 'static, // Synced to use FloatScalar tools cleanly
     V::Quantized: Hash + Eq + Copy + GridKey,
{
    pub fn new() -> Self {
        Self { 
            hud_state: Arc::new(Mutex::new(HudState::default())),
            _marker: PhantomData,
        }
    }
 

    pub fn environment() -> ParticleEnvironment<V, FluidCollisionFlags> {
        // --- 1. CLOCK & ITERATION SETTINGS ---
        let substep_count: u64 = 6;
        let collision_iterations: u64 = 2;
         let max_particles: usize = 16000;  
         
        // --- 2. PHYSICS SIZING & MATERIAL CONSTRAINTS ---
        let cell_size = V::Scalar::from_f64(0.0);
        let gravity_force = V::from_f64_array([0.0, -1600.0]);

        let space = GridSpace::new(cell_size);
        
        // --- 3. HARDWARE SPEED CONST TUNING CONFIGURATIONS ---
        let tuning = SimulationTuning::new(
            60.0,
            substep_count, 
            collision_iterations, 
            max_particles,
            cell_size, 
            V::Scalar::from_f64(0.3), 
            V::Scalar::from_f64(0.5)
        );
         
        // --- 4. ENGINE RUNTIME VISUALIZATION ASSETS ---
        let state = State::new(&Color::WATER_OCEANIC);
        let gravity = GravityModel::Constant(gravity_force); 
        
        ParticleEnvironment::new(space, tuning, state, gravity)
    }
}

// =========================================================================
// SCENE TRAIT ORCHESTRATION PIPELINE DEFINITIONS
// =========================================================================

impl<V: Vector + 'static> Scene for WaterFountainParticleSoaVerletScene<V> 
where
    V::Scalar: FloatScalar + 'static,  
   V::Quantized: Eq + Hash + Copy + GridKey, 
    SoaSimulationRenderer<VerletParticle<V>>: SimulationRenderer<VerletParticleSoaVecStorage<V>>,
    V: From<(f64, f64)>, 
{
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        // 🚀 FIXED: Added explicit vector parameter context typing to prevent inference errors
        let env = WaterFountainParticleSoaVerletScene::<V>::environment(); 
         
         
        // 🚀 FIXED: Synced argument positions to match your constructor definition perfectly
        let stream = Stream::new(
            20,                                      // start_tick
            2,                                       // ticks_per_spawn
            12,                                      // droplets_per_burst (4 wide x 3 high)
            V::from_f64_array([0.5, 0.0]),           // relative_position
            V::from_f64_array([0.0, 1700.0]),        // velocity
            V::Scalar::from_f64(2.0),                // radius
            V::Scalar::from_f64(1.0),                // density
        );

       
        
        // 🟢 COMPILES FLAWLESSLY:
        // The type parameter checking completely satisfies the solver's sorting predicates!
        let sim = Simulation::new(
           env.tuning.hz,
            VerletParticleSoaVecStorage::<V>::new(env.tuning.max_particles),
            VerletSoaGravitySolver::new(),
            FountainLifecycle::new(stream),
            env,
        );    
        
        let particle_renderer = SoaSimulationRenderer::<VerletParticle<V>>::new();   
        
        // Build and append your rendering layers to the active WGPU viewport graphics matrix
        let pass = SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone());
        renderer.add_pass(pass);
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        // Handled cleanly at the application pipeline layer
    }
}