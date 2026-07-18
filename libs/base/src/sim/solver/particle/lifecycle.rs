use crate::{math::{Vector, FloatScalar}, 
sim::{lifecycle::stream_config::StreamConfig, 
    solver::particle::{environment::ParticleEnvironment, verlet_particle::VerletParticle}, storage::CpuStorage}};


 

pub fn tick_stream_lifecycle<V, S>(
    config: &mut StreamConfig<V>,  
    storage: &mut S,
    tick: u64,
    step_dt: f64,
    environment: &ParticleEnvironment<V>,
) where
    V: Vector + 'static,
    V::Scalar: 'static, 
    S: CpuStorage<Item = VerletParticle<V>>, // 👈 Constrain the associated type here
{
    if config.should_spawn(tick) {
        let position = config.get_spawn_position(&environment.space.bounds);
        let velocity = config.velocity.clone();
        let radius = config.radius.clone(); 
        let density = config.density.clone();
        let color = config.get_color();

        let mut position2 = position;  
        position2.as_slice_mut()[1] -= radius * V::Scalar::from_f64(3.0);

        let mut position3 = position; 
        position3.as_slice_mut()[1] -= radius * V::Scalar::from_f64(6.0);

        let mut position4 = position; 
        position4.as_slice_mut()[1] -= radius * V::Scalar::from_f64(9.0);
        
        // 🟢 FIXED: Clone velocity for the first particle so it remains available for the second
        let p1 = VerletParticle::new(position)
            .with_velocity(velocity.clone(), step_dt)
            .with_radius(radius, density)
            .with_color(color);

        let p2 = VerletParticle::new(position2)
            .with_velocity(velocity, step_dt)
            .with_radius(radius, density)
            .with_color(color);

         let p3 = VerletParticle::new(position3)
            .with_velocity(velocity, step_dt)
            .with_radius(radius, density)
            .with_color(color);

        let p4 = VerletParticle::new(position4)
            .with_velocity(velocity, step_dt)
            .with_radius(radius, density)
            .with_color(color);

        storage.push(p1);
        storage.push(p2);
        storage.push(p3);
        storage.push(p4);
        config.particle_count = storage.len();
    }
}


 