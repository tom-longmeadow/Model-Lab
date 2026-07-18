use base::{math::{Vector,FloatScalar} ,sim::{lifecycle::stream_config::StreamConfig, 
    solver::particle::{environment::{GravityModel, ParticleEnvironment}, 
    runtime::RuntimeState, space::GridSpace, tuning::SimulationTuning, verlet_particle::VerletParticle}, storage::CpuStorage}, ui::layout::color::Color};
use std::hash::Hash;

pub struct WaterFountainSceneConfig;
impl WaterFountainSceneConfig{

    const PARTICLE_RADIUS: f64 =  1.2;  

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
        let substep_count: u64 = 6;
        let collision_iterations: u64 = 2;

        
        let cell_size = (Self::PARTICLE_RADIUS + Self::PARTICLE_RADIUS + Self::PARTICLE_RADIUS).into();  
        let gravity_force= V::from_f64_array([0.0, -1000.0]);

        let space = GridSpace::new(cell_size);
        let tuning=  SimulationTuning::new(substep_count, collision_iterations, 
            cell_size, 0.3.into());
        

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
        let max_particles: usize = 13000;  
        let particle_initial_velocity  = V::from_f64_array([0.0, 1500.0]);
        let particle_relative_location = V::from_f64_array([0.5, 0.0]);
        let lifecycle_start_tick: u64 = 20;
        let lifecycle_ticks_per_spawn: u64 = 2;   
        let particle_radius = Self::PARTICLE_RADIUS.into();  
        let particle_colors: &'static [Color] = &Color::WATER;

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

pub fn water_fountain_lifcycle_tick<V, S>(
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
        let jitter = environment.state.runtime_jitter * V::Scalar::from_f64(0.25);
        let position = config.get_spawn_position(&environment.space.bounds) + jitter;
        let velocity = config.velocity.clone();
        let radius = config.radius.clone(); 
        let density = config.density.clone();
        let color = Color::WHITE;//config.get_color();
        let pos = radius * V::Scalar::from_f64(2.1); 
 

        let mut mypos = position;
        for c in -2..3 { 
            mypos.as_slice_mut()[1] = position.as_slice()[1]; 
            mypos.as_slice_mut()[0] = position.as_slice()[0] + pos * V::Scalar::from_f64(c as f64);
            
            if c == 0 {
                mypos.as_slice_mut()[1] += pos + pos; 
            }
            // if c == 1 || c == -1 {
            //     mypos.as_slice_mut()[1] += pos; 
            // }

            for _ in 0..6 { 
                storage.push(VerletParticle::new(mypos)
                        .with_velocity(velocity + jitter, step_dt)
                        .with_radius(radius, density)
                        .with_color(color));

                mypos.as_slice_mut()[1] += pos; 
            }
              
        }

        
        // let mut mypos = position;
        // for mut t in 0..4 { 

        //     mypos.as_slice_mut()[0] = position.as_slice()[0]; 
        //     storage.push(VerletParticle::new(mypos)
        //             .with_velocity(velocity + jitter, step_dt)
        //             .with_radius(radius, density)
        //             .with_color(color));

        //     for _ in 0..t {  

        //         mypos.as_slice_mut()[0] += pos;  
        //         storage.push(VerletParticle::new(mypos)
        //             .with_velocity(velocity + jitter, step_dt)
        //             .with_radius(radius, density)
        //             .with_color(color));
        //     }

        //     mypos.as_slice_mut()[0] = position.as_slice()[0];
        //     for _ in 0..t {  

        //          mypos.as_slice_mut()[0] -= pos; 

        //          storage.push(VerletParticle::new(mypos)
        //             .with_velocity(velocity + jitter, step_dt)
        //             .with_radius(radius, density)
        //             .with_color(color));
        //     }

        //     mypos.as_slice_mut()[1] += pos; 
        // } 
 
        config.particle_count = storage.len();
    }
}