use base::{math::{Bounds, DVec2}, sim::{
    simulation::Simulation, solver::constraint::Insets, storage::CpuStorage
}, ui::layout::color::Color};
use crate::simulation::verlet_2d::{
    lifecycle::stream::StreamLifecycle,
    solver::gravity::GravitySolver,
    aos_vec_storage::AosVecStorage,
};

/// A simple 2D verlet simulation that spawns one particle at the origin.
pub type Verlet2dGravitySimulation = Simulation<AosVecStorage, GravitySolver, StreamLifecycle>;

/// Creates a new SimpleSim running at 60hz.
pub fn new_verlet2d_gravity_sim(hz: f64, substep_count: u64, collision_iterations: u64, 
        sim_bounds: Bounds, insets: Insets, gravity: f64,  
        start_tick: u64, ticks_per_spawn: u64,  max_particles: usize, 
        velocity: DVec2, radius: f64,  colors: Vec<Color>) -> Verlet2dGravitySimulation {
  
    Simulation::new(
        hz,
        <AosVecStorage as CpuStorage>::new(max_particles),
        GravitySolver::new(substep_count, collision_iterations, gravity, insets),
        StreamLifecycle::new(start_tick, ticks_per_spawn, max_particles,  velocity, radius, colors),
        sim_bounds,
    )
}
