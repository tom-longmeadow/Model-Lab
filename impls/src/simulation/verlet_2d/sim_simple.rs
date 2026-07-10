use base::sim::{
    simulation::Simulation,
    storage::CpuStorage,
    Bounds,
};
use crate::simulation::verlet_2d::{
    lifecycle::simple::SimpleLifecycle,
    solver::gravity::GravitySolver,
    aos_vec_storage::AosVecStorage,
};

/// A simple 2D verlet simulation that spawns one particle at the origin.
pub type Verlet2dGravitySimulation = Simulation<AosVecStorage, GravitySolver, SimpleLifecycle>;

/// Creates a new SimpleSim running at 60hz.
pub fn new_verlet2d_gravity_sim(hz: f64, sim_bounds: Bounds, gravity: f64, restitution: f64) -> Verlet2dGravitySimulation {
    Simulation::new(
        hz,
        <AosVecStorage as CpuStorage>::new(1),
        GravitySolver::new(sim_bounds, restitution, gravity),
        SimpleLifecycle::new(),
        sim_bounds,
    )
}
