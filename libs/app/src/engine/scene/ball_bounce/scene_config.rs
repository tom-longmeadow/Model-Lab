use base::{math::Vector, sim::{lifecycle::stream_config::StreamConfig, 
    solver::particle::{environment::{GravityModel, ParticleEnvironment}, 
    runtime::RuntimeState, space::GridSpace, tuning::SimulationTuning, verlet_particle::VerletParticle}, storage::CpuStorage}, ui::layout::color::Color};
use std::hash::Hash;

pub struct BallBounceSceneConfig;
impl BallBounceSceneConfig{

    const PARTICLE_RADIUS: f64 =  10.0;  

    pub fn hz() -> f64
    {
        60.0 
    }
  
    pub fn environment<V>() -> ParticleEnvironment<V> 
    where 
        V: Vector, 
        V::Scalar: From<f64>, 
        V::Quantized: Eq + Hash, 
    {
        let substep_count: u64 = 8;
        let collision_iterations: u64 = 2;

        
        let cell_size = Self::PARTICLE_RADIUS.into();  
        let gravity_force= V::from_f64_array([0.0, -1600.0]);

        let space = GridSpace::new(cell_size);
        let tuning=  SimulationTuning::new(substep_count, collision_iterations, 
            cell_size, 0.8.into());
        

        let state=  RuntimeState::new();
        let gravity = GravityModel::Constant(gravity_force);
        let env = ParticleEnvironment::new(space, tuning, state, gravity);
        env 
    } 
 
    pub fn config<V>() -> StreamConfig<V> 
    where 
        V: Vector, 
        V::Scalar: From<f64>, 
        V::Quantized: Eq + Hash, 
    {
        let max_particles: usize = 600;  
        let particle_initial_velocity  = V::from_f64_array([2600.0, -800.0]);
        let particle_relative_location = V::from_f64_array([0.2, 1.0]);
        let lifecycle_start_tick: u64 = 20;
        let lifecycle_ticks_per_spawn: u64 = 1;   
        let particle_radius = Self::PARTICLE_RADIUS.into();  
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

        stream_config
    } 
}


pub fn ball_bounce_lifcycle_tick<V, S>(
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

        // let mut position2 = position;  
        // position2.as_slice_mut()[1] -= radius * V::Scalar::from_f64(3.0);

        // let mut position3 = position; 
        // position3.as_slice_mut()[1] -= radius * V::Scalar::from_f64(6.0);

        // let mut position4 = position; 
        // position4.as_slice_mut()[1] -= radius * V::Scalar::from_f64(9.0);
        
        // 🟢 FIXED: Clone velocity for the first particle so it remains available for the second
        let p1 = VerletParticle::new(position)
            .with_velocity(velocity.clone(), step_dt)
            .with_radius(radius, density)
            .with_color(color);

        // let p2 = VerletParticle::new(position2)
        //     .with_velocity(velocity, step_dt)
        //     .with_radius(radius, density)
        //     .with_color(color);

        //  let p3 = VerletParticle::new(position3)
        //     .with_velocity(velocity, step_dt)
        //     .with_radius(radius, density)
        //     .with_color(color);

        // let p4 = VerletParticle::new(position4)
        //     .with_velocity(velocity, step_dt)
        //     .with_radius(radius, density)
        //     .with_color(color);

        storage.push(p1);
        // storage.push(p2);
        // storage.push(p3);
        // storage.push(p4);
        config.particle_count = storage.len();
    }
}

