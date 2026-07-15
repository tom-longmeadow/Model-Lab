 use crate::{sim::{clock::Clock, lifecycle::Lifecycle, metrics::SimMetrics, solver::Solver, storage::Storage} };

pub trait Simulate {
    type Storage: Storage;  
    type Environment;  

    fn simulate(&mut self, frame_time: f64);
    fn storage(&self) -> &Self::Storage;
    fn metrics(&self) -> &SimMetrics;   
    fn environment(&mut self) -> &mut Self::Environment;
}

pub trait SubstepProvider {
    fn substep_count(&self) -> u64;
}

/// Owns and coordinates the three components of a simulation.
/// Makes no assumptions about physics method, dimensionality,
/// memory layout, or the type of simulation.
pub struct Simulation<St, Sv, Lc, Env>
where
    St: Storage,
    Sv: Solver<St, Env>,
    Lc: Lifecycle<St, Env>,
    Env: SubstepProvider,
{
    storage:     St,
    solver:      Sv,
    lifecycle:   Lc,
    clock:       Clock,
    metrics:     SimMetrics,
    environment: Env,   
}

impl<St, Sv, Lc, Env> Simulation<St, Sv, Lc, Env> 
where
    St: Storage,
    Sv: Solver<St, Env>,
    Lc: Lifecycle<St, Env>,
    Env: SubstepProvider,
{
    /// Creates a new simulation running at `hz` ticks per second.
    /// Calls `lifecycle.init` and `solver.init`.
    pub fn new(hz: f64, storage: St, solver: Sv, lifecycle: Lc, environment: Env) -> Self {
        let mut sim = Self { 
            storage, 
            solver, 
            lifecycle, 
            clock: Clock::new(hz), 
            metrics: SimMetrics::default(), 
            environment,
        };
        sim.metrics.hz = hz;
        sim.solver.init(&mut sim.storage, &mut sim.environment);
        sim
    }
 
    /// Read access to the clock — for alpha, elapsed time, tick count.
    pub fn clock(&self) -> &Clock { 
        &self.clock 
    }
}

// 🚀 Added `Env: SubstepProvider` to this impl block so `self.environment` can be queried
impl<St, Sv, Lc, Env> Simulate for Simulation<St, Sv, Lc, Env>
where
    St: Storage,
    Sv: Solver<St, Env>,
    Lc: Lifecycle<St, Env>,
    Env: SubstepProvider, 
{
    type Storage = St;   
    type Environment = Env; 

    /// Advances the simulation by `frame_time` seconds of real-world time.
    /// May run zero, one, or many ticks depending on the accumulator.
    fn simulate(&mut self, frame_time: f64) {
        let tick  = self.clock.tick();
        let steps = self.clock.advance(frame_time);
        
        // 🚀 Pull substeps cleanly from the environment now!
        let subs  = self.environment.substep_count().max(1);
        
        let step_dt = self.clock.fixed_dt();
        let sub_dt  = step_dt / subs as f64;
        let storage_size: usize = self.storage.len();

        let physics_start = std::time::Instant::now();

        for step in 0..steps {
            let current_tick = tick + step as u64;

            self.storage.pre_step();
            self.lifecycle.tick(&mut self.storage, current_tick, &self.environment);

            self.solver.pre_step(&mut self.storage, step_dt, current_tick, &mut self.environment);
            for _ in 0..subs { 
                self.solver.sub_step(&mut self.storage, sub_dt, &self.environment);
            }
            self.solver.post_step(&mut self.storage, step_dt, &self.environment);

            self.storage.post_step();
        }

        let total_frame_ns = physics_start.elapsed().as_nanos();

        self.metrics.storage_size = storage_size;
        self.metrics.total_ticks      = self.clock.tick();
        self.metrics.accumulator_ms   = self.clock.accumulator() * 1000.0; 
        self.metrics.step_time_ms = if steps > 0 {
            (total_frame_ns as f64 / steps as f64) / 1_000_000.0
        } else {
            0.0
        }; 
    }
    
    fn storage(&self) -> &Self::Storage {
        &self.storage
    }

    fn metrics(&self) -> &SimMetrics {
        &self.metrics
    }

    #[inline]
    fn environment(&mut self) -> &mut Env {  
        &mut self.environment
    }
}

// pub trait Simulate {
//     type Storage: Storage;  

//     fn simulate(&mut self, frame_time: f64);
//     fn storage(&self) -> &Self::Storage;
//     fn metrics(&self) -> &SimMetrics;  
// }

// pub trait SubstepProvider {
//     fn substep_count(&self) -> u64;
// }

// /// Owns and coordinates the three components of a simulation.
// /// Makes no assumptions about physics method, dimensionality,
// /// memory layout, or the type of simulation.
// pub struct Simulation<St, Sv, Lc, Env>
// where
//     St: Storage,
//     Sv: Solver<St, Env>,
//     Lc: Lifecycle<St, Env>,
//     Env: SubstepProvider,
// {
//     storage:   St,
//     solver:    Sv,
//     lifecycle: Lc,
//     clock:     Clock,
//     metrics:   SimMetrics,
//     environment: Env,   
// }

// impl<St, Sv, Lc, Env> Simulation<St, Sv, Lc, Env> 
// where
//     St: Storage,
//     Sv: Solver<St, Env>,
//     Lc: Lifecycle<St, Env>,
//     Env: SubstepProvider,
// {
//     /// Creates a new simulation running at `hz` ticks per second.
//     /// Calls `lifecycle.init` and `solver.init`.
//     pub fn new(hz: f64, storage: St, solver: Sv, lifecycle: Lc, environment: Env) -> Self {//, bounds: Sv::Bounds) -> Self {
//         let mut sim = Self { 
//             storage, 
//             solver, 
//             lifecycle, 
//             clock: Clock::new(hz), 
//             metrics: SimMetrics::default(), 
//             environment,
            
//             //bounds 
//         };
//         sim.metrics.hz = hz;
//         sim.solver.init(&mut sim.storage);
//         sim
//     }
 
//     /// Read access to the clock — for alpha, elapsed time, tick count.
//     pub fn clock(&self) -> &Clock { 
//         &self.clock 
//     }

//     // /// Read access to simulation spatial bounds.
//     // pub fn bounds(&self) -> &Sv::Bounds { 
//     //     &self.bounds 
//     // }
// }

// impl<St, Sv, Lc, Env> Simulate for Simulation<St, Sv, Lc, Env>
// where
//     St: Storage,
//     Sv: Solver<St, Env>,
//     Lc: Lifecycle<St, Env>,
// {
//     type Storage = St;   

//     /// Advances the simulation by `frame_time` seconds of real-world time.
//     /// May run zero, one, or many ticks depending on the accumulator.
//     fn simulate(&mut self, frame_time: f64) {
//         let tick  = self.clock.tick();
//         let steps = self.clock.advance(frame_time);
//         let subs  = self.solver.substep_count().max(1);
//         let step_dt = self.clock.fixed_dt();
//         let sub_dt  = step_dt / subs as f64;
//         let storage_size: usize = self.storage.len();

//         // Start timing everything here
//         let physics_start = Instant::now();

//         for step in 0..steps {
//             let current_tick = tick + step as u64;

//             self.storage.pre_step();
//             self.lifecycle.tick(&mut self.storage, current_tick, &self.environment);

//             self.solver.pre_step(&mut self.storage, step_dt, current_tick, &self.environment);
//             for _ in 0..subs {
//                 self.solver.sub_step(&mut self.storage, sub_dt);
//             }
//             self.solver.post_step(&mut self.storage, step_dt);

//             self.storage.post_step();
//         }

//         // Single measurement for the whole frame
//         let total_frame_ns = physics_start.elapsed().as_nanos();

//         self.metrics.storage_size = storage_size;
//         self.metrics.total_ticks      = self.clock.tick();
//         self.metrics.accumulator_ms   = self.clock.accumulator() * 1000.0; 
//         self.metrics.step_time_ms = if steps > 0 {
//             (total_frame_ns as f64 / steps as f64) / 1_000_000.0
//         } else {
//             0.0
//         }; 
//     }
    
//     fn storage(&self) -> &Self::Storage {
//         &self.storage
//     }

//     fn metrics(&self) -> &SimMetrics {
//         &self.metrics
//     }

//     // fn set_bounds(&mut self, bounds: Self::Bounds) {
//     //     self.bounds = bounds;
//     // }
// }

/********************/
/*      TESTS       */
/********************/

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{aabb::AABB, math::Vec2, sim::{
//         lifecycle::Lifecycle,
//         storage::{AosCpuStorage, CpuStorage, Storage},
//     }};

//     // --- mock entity ---
//     #[derive(Default, Debug, Clone, Copy, PartialEq)]
//     pub struct MockEntity {
//         pub d64: f64,
//         pub c8: u8,
//     }

//      // --- mock storage ---
//     #[derive(Clone)]
//     pub struct MockStorage {
//         data: Vec<MockEntity>,
//         capacity: usize,
//     }

//     impl MockStorage {
//         pub fn new(capacity: usize) -> Self {
//             Self {
//                 data: Vec::with_capacity(capacity),
//                 capacity,
//             }
//         }
//     }

//     impl Storage for MockStorage {
//         fn pre_step(&mut self) {}
//         fn post_step(&mut self) {}
//         fn len(&self) -> usize {
//             self.data.len()
//         }
//         fn capacity(&self) -> usize {
//             self.capacity
//         }
        
//         fn clear(&mut self) {
//             self.data.clear(); 
//         }  
//     }

//     impl CpuStorage for MockStorage {
//         fn new(_capacity: usize) -> Self {
//             todo!()
//         }
//     }

//     impl AosCpuStorage for MockStorage {
//         type Item = MockEntity;

//         fn as_slice(&self) -> &[Self::Item] {
//             &self.data
//         }

//         fn as_slice_mut(&mut self) -> &mut [Self::Item] {
//             &mut self.data
//         }

//         fn push(&mut self, item: Self::Item) {
//             self.data.push(item);
//         }

      
//         fn swap_remove(&mut self, _index: usize) -> Self::Item {
//             todo!()
//         }
//     }

//     // --- mock solver ---
//     pub struct MockSolver {
//         pub calls: String,
//         pub received_ticks: Vec<u64>,
//         pub received_dts: Vec<f64>,
//         pub iteration_count: u64,
//     }

//     impl MockSolver {
//         pub fn new() -> Self {
//             Self {
//                 calls: String::new(),
//                 received_ticks: Vec::new(),
//                 received_dts: Vec::new(),
//                 iteration_count: 2,
//             }
//         }
//     }

//     impl<S> Solver<S> for MockSolver
//     where
//         S: Storage + AosCpuStorage<Item = MockEntity>,
//     {
//         // Fix: Explicitly declare the boundary type this solver uses
//         type Bounds = AABB<Vec2>;

//         fn substep_count(&self) -> u64 {
//             self.iteration_count
//         }

//         // Fix: Update the argument type from a raw `&AABB` to `&Self::Bounds`
//         fn pre_step(&mut self, storage: &mut S, _: f64, tick: u64, _bounds: &Self::Bounds) {
//             self.calls.push_str("prepare-");
//             self.received_ticks.push(tick);
//             storage.push(setup_entity1());
//         }

//         fn sub_step(&mut self, _: &mut S, dt: f64) {
//             self.calls.push_str("sub-");
//             self.received_dts.push(dt);
//         }

//         fn post_step(&mut self, _: &mut S, _: f64) {
//             self.calls.push_str("finalize");
//         }
//     }

   
//    pub struct MockLifecycle;  
//     impl<S> Lifecycle<S> for MockLifecycle 
//     where 
//         S: Storage 
//     {
//         // 1. Explicitly satisfy the missing associated type requirement
//         type Bounds = AABB<Vec2>;

//         // 2. Update the argument to use Self::Bounds (or your concrete AABB<Vec2> type)
//         fn tick(&mut self, _storage: &mut S, _tick: u64, _bounds: &Self::Bounds) {
//             // don't do anything
//         }
//     }

//     // --- Helpers ---
//     fn setup_sim() -> Simulation<MockStorage, MockSolver, MockLifecycle> {
//         // Build the explicit spatial structures for the test environment
//         let min_corner = Vec2::new(0.0, 0.0);
//         let max_corner = Vec2::new(1000.0, 1000.0);
//         let test_bounds = AABB::new(min_corner, max_corner);

//         Simulation::new(
//             100.0,
//             MockStorage::new(10),
//             MockSolver::new(),
//             MockLifecycle,
//             test_bounds
//         )
//     }

//     fn setup_entity1() -> MockEntity {
//         MockEntity { d64: 1.2, c8: 5 }
//     }

//     // --- tests ---
//     #[test]
//     fn test_initialization() {
//         let sim = setup_sim();
//         assert_eq!(sim.storage().capacity(), 10);
//         assert_eq!(sim.clock().elapsed_time(), 0.0);
//         assert_eq!(sim.clock().fixed_dt(), 0.01);
//         assert_eq!(sim.clock().tick(), 0);
//     }

//     #[test]
//     fn test_sim_step_execution() {
//         let mut sim = setup_sim();
//         let e = setup_entity1();

//         sim.simulate(0.01);

//         assert_eq!(sim.solver.calls, "prepare-sub-sub-finalize");
//         assert_eq!(sim.storage().capacity(), 10);
//         assert_eq!(sim.storage().len(), 1);
//         assert_eq!(sim.storage().as_slice()[0], e);
//     }

//     #[test]
//     fn test_tick_sequence() {
//         let mut sim = setup_sim();
//         let dt = sim.clock().fixed_dt();

//         sim.simulate(dt);
//         assert_eq!(sim.clock().tick(), 1);
//         assert_eq!(sim.clock().elapsed_time(), dt);

//         sim.simulate(dt * 2.0);
//         assert_eq!(sim.clock().tick(), 3);
//         assert_eq!(sim.clock().elapsed_time(), dt * 3.0);
//     }

//     #[test]
//     fn test_zero_iterations_safety() {
//         let mut sim = setup_sim();
//         let dt = sim.clock().fixed_dt();

//         sim.simulate(0.0);
//         assert_eq!(sim.clock().tick(), 0);
//         assert_eq!(sim.clock().elapsed_time(), 0.0);

//         sim.solver.iteration_count = 0;
//         sim.simulate(dt);
//         assert_eq!(sim.clock().tick(), 1);
//         assert_eq!(sim.clock().elapsed_time(), dt);
//     }

//     #[test]
//     fn test_tick_increment_across_multiple_steps() {
//         let mut sim = setup_sim();
//         let dt = sim.clock().fixed_dt();

//         sim.simulate(dt * 2.0);
//         assert_eq!(sim.solver.received_ticks, vec![0, 1]);

//         sim.simulate(dt * 1.5);
//         assert_eq!(sim.solver.received_ticks, vec![0, 1, 2]);

//         sim.simulate(dt * 0.5);
//         assert_eq!(sim.solver.received_ticks, vec![0, 1, 2, 3]);
//     }

//     #[test]
//     fn test_substep_dt_scaling() {
//         let mut sim = setup_sim();
//         let count = sim.solver.iteration_count;
//         let dt_step = sim.clock().fixed_dt();
//         let dt_sub = dt_step / count as f64;

//         sim.simulate(dt_step * 4.0);
//         assert_eq!(sim.solver.received_dts, vec![dt_sub; (count * 4) as usize]);
//     }

//     #[test]
//     fn test_storage_mutation_count() {
//         let mut sim = setup_sim();
//         let dt_step = sim.clock().fixed_dt();
//         sim.simulate(dt_step * 2.1);
//         assert_eq!(sim.storage().len(), 2);
//     }
// }