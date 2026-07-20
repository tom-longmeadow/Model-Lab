use std::{marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;
use base::{
    math::Vector, 
    sim::{ 
        simulation::Simulation, 
        solver::particle::{
            environment::{GravityModel, ParticleEnvironment}, flags::FluidCollisionFlags, lifecycle::{Stream, ball::BallLifecycle}, space::{GridSpace, grid_key::GridKey}, state::State, tuning::SimulationTuning, verlet_particle::VerletParticle, verlet_soa_gravity_solver::VerletSoaGravitySolver, verlet_soa_vec_storage::VerletParticleSoaVecStorage 
        },  
    }, ui::layout::color::Color,  
};
use base::math::FloatScalar;
 
use crate::{
    engine::{input::InputState, scene::{Scene,   }},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState}, simulation::{pass::SimulationPass, renderer::SimulationRenderer, soa::SoaSimulationRenderer}, 
    }, 
};


 
pub struct BallBounceParticleSoaVerletScene<V: Vector> { 
    hud_state: Arc<Mutex<HudState>>,
    _marker: PhantomData<V>, 
}


impl<V: Vector + 'static> BallBounceParticleSoaVerletScene<V> 
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
        let substep_count: u64 = 8;
        let collision_iterations: u64 = 2;
        let max_particles: usize = 600;  
         
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
        let state = State::new(&Color::RAINBOW);
        let gravity = GravityModel::Constant(gravity_force); 
        
        ParticleEnvironment::new(space, tuning, state, gravity)
    }
}
 

// =========================================================================
// SCENE TRAIT ORCHESTRATION PIPELINE DEFINITIONS
// =========================================================================

impl<V: Vector + 'static> Scene for BallBounceParticleSoaVerletScene<V> 
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

          
        let env = BallBounceParticleSoaVerletScene::<V>::environment(); 
         
         
         
        let stream = Stream::new(
            20,                                      // start_tick
            2,                                       // ticks_per_spawn
            1,                                      // droplets_per_burst (4 wide x 3 high)
            V::from_f64_array([0.2, 0.9]),           // relative_position
            V::from_f64_array([2000.0, 0.0]),        // velocity
            V::Scalar::from_f64(10.0),                // radius
            V::Scalar::from_f64(1.0),                // density
        );

        

        // 🟢 COMPILES FLAWLESSLY:
        // Changed CpuStorage::new invocation to direct struct instantiation pattern to match WaterFountain
        let sim = Simulation::new(
           env.tuning.hz,
            VerletParticleSoaVecStorage::<V>::new(env.tuning.max_particles),
            VerletSoaGravitySolver::new(),
            BallLifecycle::new(stream),
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


//     pub fn environment<V>() -> ParticleEnvironment<V> 
//     where 
//         V: Vector, 
//         V::Scalar: From<f64>, 
//         V::Quantized: Eq + Hash, 
//     {
//         let substep_count: u64 = 8;
//         let collision_iterations: u64 = 2;

        
//         let cell_size = Self::PARTICLE_RADIUS.into();  
//         let gravity_force= V::from_f64_array([0.0, -1600.0]);

//         let space = GridSpace::new(cell_size);
//         let tuning=  SimulationTuning::new(substep_count, collision_iterations, 
//             cell_size, 0.4.into(), 0.2.into());
        

//         let state=  RuntimeState::new();
//         let gravity = GravityModel::Constant(gravity_force);
//         let env = ParticleEnvironment::new(space, tuning, state, gravity);
//         env 
//     } 
 
//     pub fn config<V>() -> StreamConfig<V> 
//     where 
//         V: Vector, 
//         V::Scalar: From<f64>, 
//         V::Quantized: Eq + Hash, 
//     {
//         let max_particles: usize = 600;  
//         let particle_initial_velocity  = V::from_f64_array([2600.0, -800.0]);
//         let particle_relative_location = V::from_f64_array([0.2, 1.0]);
//         let lifecycle_start_tick: u64 = 20;
//         let lifecycle_ticks_per_spawn: u64 = 1;   
//         let particle_radius = Self::PARTICLE_RADIUS.into();  
//         let particle_colors: &'static [Color] = &Color::RAINBOW;

//         let stream_config = StreamConfig::<V>::new(
//             lifecycle_start_tick, 
//             lifecycle_ticks_per_spawn, 
//             max_particles, 
//             particle_relative_location,
//             particle_initial_velocity, 
//             particle_radius, 
//             particle_colors
//         );

//         stream_config
//     } 
// }


// pub fn ball_bounce_lifcycle_tick<V, S>(
//     config: &mut StreamConfig<V>,  
//     storage: &mut S,
//     tick: u64,
//     step_dt: f64,
//     environment: &ParticleEnvironment<V>,
// ) where
//     V: Vector + 'static,
//     V::Scalar: 'static, 
//     S: CpuStorage<Item = VerletParticle<V>>, // 👈 Constrain the associated type here
// {
//     if config.should_spawn(tick) {
//         let position = config.get_spawn_position(&environment.space.bounds);
//         let velocity = config.velocity.clone();
//         let radius = config.radius.clone(); 
//         let density = config.density.clone();
//         let color = config.get_color();

//         // let mut position2 = position;  
//         // position2.as_slice_mut()[1] -= radius * V::Scalar::from_f64(3.0);

//         // let mut position3 = position; 
//         // position3.as_slice_mut()[1] -= radius * V::Scalar::from_f64(6.0);

//         // let mut position4 = position; 
//         // position4.as_slice_mut()[1] -= radius * V::Scalar::from_f64(9.0);
        
//         // 🟢 FIXED: Clone velocity for the first particle so it remains available for the second
//         let p1 = VerletParticle::new(position)
//             .with_velocity(velocity.clone(), step_dt)
//             .with_radius(radius, density)
//             .with_color(color);

//         // let p2 = VerletParticle::new(position2)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         //  let p3 = VerletParticle::new(position3)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         // let p4 = VerletParticle::new(position4)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         storage.push(p1);
//         // storage.push(p2);
//         // storage.push(p3);
//         // storage.push(p4);
//         config.particle_count = storage.len();
//     }
// }


// pub struct BallBounceParticleSoaVerletScene<V: Vector> { 
//     hud_state: Arc<Mutex<HudState>>,
//     _marker: PhantomData<V>, 
// }

// impl<V: Vector> BallBounceParticleSoaVerletScene<V> {
//     pub fn new() -> Self {
//         Self { 
//             hud_state: Arc::new(Mutex::new(HudState::default())),
//             _marker: PhantomData,
//         }
//     }
// }

// impl<V: Vector + 'static> Scene for BallBounceParticleSoaVerletScene<V> 
// where
//     V::Scalar: From<f64>, 
//     V::Quantized: Hash + Eq,  
//     SoaSimulationRenderer<VerletParticle<V>>: SimulationRenderer<VerletParticleSoaVecStorage<V>>,
//     V: From<(f64, f64)>, 
// {
//     fn build_passes(&mut self, renderer: &mut GraphicsContext) {
//         if renderer.pass_count() > 0 {
//             return;
//         }

//         let hz = BallBounceSceneConfig::hz();
//         let env = BallBounceSceneConfig::environment();
//         let config = BallBounceSceneConfig::config();
//         let sim = Simulation::new(
//             hz,
//             <VerletParticleSoaVecStorage<V> as CpuStorage>::new(config.max_particles),
//             VerletSoaGravitySolver::new( config.max_particles, V::Scalar::ONE , false),
//             BallBounceSoaStreamLifecycle::<V>::new(config),
//             env,
//         );    
//         let particle_renderer = SoaSimulationRenderer::<VerletParticle<V>>::new();   
//         let pass = SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone());
//         renderer.add_pass(pass);
//         renderer.add_pass(HudPass::new(self.hud_state.clone()));
//     }

//     fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
//         // Update logic remains clean
//     }
// }