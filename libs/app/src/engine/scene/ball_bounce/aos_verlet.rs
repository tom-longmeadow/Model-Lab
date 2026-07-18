use std::{marker::PhantomData, sync::{Arc, Mutex}};
 use std::hash::Hash;
use base::{math::Vector, 
    sim::{
        simulation::Simulation, 
        solver::particle::{ 
        verlet_aos_gravity_solver::VerletAosGravitySolver, 
        verlet_aos_vec_storage::VerletParticleAosVecStorage, 
        verlet_particle::VerletParticle}, storage::CpuStorage
    }
};
 
use crate::{
    engine::{input::InputState, scene::{Scene, ball_bounce::{scene_config::BallBounceSceneConfig, verlet_aos_stream_lifecycle::BallBounceAosStreamLifecycle}}},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState},
        simulation::{aos::AosSimulationRenderer, pass::SimulationPass, renderer::SimulationRenderer},
    }, 
};
use base::math::FloatScalar;


pub struct BallBounceParticleAosVerletScene<V: Vector> { 
    hud_state: Arc<Mutex<HudState>>,
    _marker: PhantomData<V>, // Necessary because V is not used in the struct fields
}

impl<V: Vector> BallBounceParticleAosVerletScene<V> {
    pub fn new() -> Self {
        Self { 
            hud_state: Arc::new(Mutex::new(HudState::default())),
            _marker: PhantomData,
        }
    }
}

impl<V: Vector + 'static> Scene for BallBounceParticleAosVerletScene<V> 
where
    V::Scalar: From<f64>, 
    V::Quantized: Hash + Eq, 
    AosSimulationRenderer<VerletParticle<V>>: SimulationRenderer<VerletParticleAosVecStorage<V>>,
    V: From<(f64, f64)>, 
{
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }
 
        let hz = BallBounceSceneConfig::hz();
        let env = BallBounceSceneConfig::environment();
        let config = BallBounceSceneConfig::config();
 
        let sim = Simulation::new(
            hz,
            <VerletParticleAosVecStorage<V> as CpuStorage>::new(config.max_particles),
            VerletAosGravitySolver::new( config.max_particles, V::Scalar::ONE ),
            BallBounceAosStreamLifecycle::<V>::new(config),
            env,
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
 

 