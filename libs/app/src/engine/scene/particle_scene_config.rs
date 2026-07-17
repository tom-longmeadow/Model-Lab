use base::{math::{Vector}, sim::{lifecycle::stream_config::StreamConfig, 
    solver::particle::{environment::{GravityModel, ParticleEnvironment}, 
    runtime::RuntimeState, space::GridSpace, tuning::SimulationTuning}}, ui::layout::color::Color};
use std::hash::Hash;


pub struct ParticleSceneConfig;
impl ParticleSceneConfig{

    pub fn hz() -> f64
    {
        60.0 
    }
 
    pub fn config<V>() -> StreamConfig<V> 
    where 
        V: Vector, 
        V::Scalar: From<f64>, 
        V::Quantized: Eq + Hash, 
    {
        let max_particles: usize = 300;  
        let particle_initial_velocity  = V::from_f64_array([4.0, -1.0]);
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

        stream_config
    }

    pub fn environment<V>() -> ParticleEnvironment<V> 
    where 
        V: Vector, 
        V::Scalar: From<f64>, 
        V::Quantized: Eq + Hash, 
    {
        let substep_count: u64 = 8;
        let collision_iterations: u64 = 4;

        let particle_radius: V::Scalar = 10.0.into();  
        let cell_size = particle_radius * 1.0.into();  
        let gravity_force= V::from_f64_array([0.0, -1600.0]);

        let space = GridSpace::new(cell_size);
        let tuning=  SimulationTuning::new(substep_count, collision_iterations, particle_radius);
        let state=  RuntimeState::new();
        let gravity = GravityModel::Constant(gravity_force);
        let env = ParticleEnvironment::new(space, tuning, state, gravity);
        env 
    } 

}