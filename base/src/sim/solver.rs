use crate::sim::storage::Storage;



/// A Simulation Solver.
pub trait Solver<S: Storage>  { 

    /// Returns how many substeps to run per frame.
    fn substep_count(&self) -> usize { 1 }

    /// Per-Step Setup (Expensive/Static)
    /// Called once per frame. Use this for heavy lifting like 
    /// rebuilding spatial hashes or grid lookups.
    /// Called once per frame before substeps.
    fn pre_step(&mut self, storage: &mut S, dt: f64, tick: u64);

    // Apply External Forces (Gravity, Wind) 
    /// Solve the step
    /// Particle; Handle Collisions and Constraints. This is where the "heavy lifting" of moving particles happens.
    /// Eulerian: Pressure Projection. This is the iterative solver that ensures the fluid is incompressible (the "Poisson" step).
    // Called `substep_count` times per frame.
    fn substep(&mut self, storage: &mut S, dt: f64);  

    /// Per-Step Cleanup (Export)
    /// Eulerian: Advection.
    /// Copy data back to the GPU for rendering 
    /// Called once per frame after substeps.
    fn post_step(&mut self, storage: &mut S);
}

 