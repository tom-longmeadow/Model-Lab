use std::{marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;
use base::{
    math::Vector, 
    sim::{ 
        simulation::Simulation, 
        solver::particle::{
            verlet_particle::VerletParticle, verlet_soa_gravity_solver::VerletSoaGravitySolver, 
            verlet_soa_stream_lifecycle::SoaStreamLifecycle, verlet_soa_vec_storage::VerletParticleSoaVecStorage 
        }, 
        storage::CpuStorage
    },  
};
 
use crate::{
    engine::{input::InputState, scene::{Scene, particle_scene_config::ParticleSceneConfig}},
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
    SoaSimulationRenderer<VerletParticle<V>>: SimulationRenderer<VerletParticleSoaVecStorage<V>>,
    V: From<(f64, f64)>, 
{
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        let hz = ParticleSceneConfig::hz();
        let env = ParticleSceneConfig::environment();
        let config = ParticleSceneConfig::config();
        let sim = Simulation::new(
            hz,
            <VerletParticleSoaVecStorage<V> as CpuStorage>::new(config.max_particles),
            VerletSoaGravitySolver::new( config.max_particles),
            SoaStreamLifecycle::<V>::new(config),
            env,
        );    
        let particle_renderer = SoaSimulationRenderer::<VerletParticle<V>>::new();   
        let pass = SimulationPass::new(sim, particle_renderer).with_hud(self.hud_state.clone());
        renderer.add_pass(pass);
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        // Update logic remains clean
    }
}