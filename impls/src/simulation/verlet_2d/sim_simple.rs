use base::sim::{
    simulation::Simulation,
    storage::CpuStorage,
};
use crate::simulation::verlet_2d::{
    lifecycle::simple::SimpleLifecycle,
    solver::simple::SimpleSolver,
    vec_storage::VecStorage,
};

/// A simple 2D verlet simulation that spawns one particle at the origin.
pub type SimpleSim = Simulation<VecStorage, SimpleSolver, SimpleLifecycle>;

/// Creates a new SimpleSim running at 60hz.
pub fn new_simple_sim() -> SimpleSim {
    Simulation::new(
        60.0,
        <VecStorage as CpuStorage>::new(1),
        SimpleSolver,
        SimpleLifecycle::new(),
    )
}
