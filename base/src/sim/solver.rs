use crate::sim::storage::Storage;
pub mod verlet;

/// Advances simulation state by consuming fixed timesteps.
/// Makes no assumptions about physics method, dimensionality,
/// or type of simulation.
pub trait Solver<S: Storage> {

    /// Called once after construction.
    /// Use to populate initial state into storage.
    fn init(&mut self, _storage: &mut S) {}

    /// Number of substeps to run per tick.
    /// Higher values improve stability at the cost of performance.
    fn substep_count(&self) -> usize { 1 }

    /// Called once per tick before substeps.
    /// Use for broad phase, sorting, or per-tick setup.
    fn pre_step(&mut self, storage: &mut S, dt: f64, tick: u64);

    /// Called `substep_count` times per tick.
    /// `dt` is already divided by substep_count.
    fn substep(&mut self, storage: &mut S, dt: f64);

    /// Called once per tick after all substeps complete.
    /// Use for post-solve corrections or diagnostics.
    fn post_step(&mut self, storage: &mut S);
}



 