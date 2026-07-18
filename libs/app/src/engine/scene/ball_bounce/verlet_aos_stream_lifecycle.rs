use base::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment, 
    verlet_aos_vec_storage::VerletParticleAosVecStorage}}};

use crate::engine::scene::ball_bounce::scene_config::ball_bounce_lifcycle_tick;

 

pub struct BallBounceAosStreamLifecycle<V: Vector> {
    pub config: StreamConfig<V>,
}

impl<V: Vector> BallBounceAosStreamLifecycle<V> {
    pub fn new(config: StreamConfig<V>) -> Self {
        Self { config }
    }
}

impl<V: Vector> Lifecycle<VerletParticleAosVecStorage<V>, ParticleEnvironment<V>> for BallBounceAosStreamLifecycle<V> 
where
     V: Vector + std::ops::Sub<Output = V> + 'static,  
    V::Scalar: 'static,
{
    fn tick(&mut self, storage: &mut VerletParticleAosVecStorage<V>, tick: u64, step_dt: f64, environment: &ParticleEnvironment<V>) {
        
        ball_bounce_lifcycle_tick(&mut self.config, storage, tick, step_dt, environment); 
    } 
}

 