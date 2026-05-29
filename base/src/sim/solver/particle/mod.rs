// pub mod verlet;

// use crate::sim::{solver::Solver, storage::Storage};

// pub trait ParticleSolver<S: Storage>: Solver<S> {
//     type Force: ForceModel<S>;
//     fn force_model(&mut self) -> &mut Self::Force;
// }


// // Classic Verlet stores pos_old instead of vel.
// pub trait ClassicVerletItem<const N: usize> {
//     fn pos(&self)         -> [f64; N];
//     fn pos_old(&self)     -> [f64; N];
//     fn acc(&self)         -> [f64; N];
//     fn set_pos(&mut self, v: [f64; N]);
//     fn set_pos_old(&mut self, v: [f64; N]);
//     fn set_acc(&mut self, v: [f64; N]);
// }

// // --- shared item trait ---
// // Velocity Verlet and Leapfrog share the same fields — only update order differs.
// pub trait NewtonianItem<const N: usize> {
//     fn pos(&self)         -> [f64; N];
//     fn vel(&self)         -> [f64; N];
//     fn acc(&self)         -> [f64; N];
//     fn set_pos(&mut self, v: [f64; N]);
//     fn set_vel(&mut self, v: [f64; N]);
//     fn set_acc(&mut self, v: [f64; N]);
// }



// // --- ForceModel trait ---
// // Computes and writes acceleration onto all particles in storage. 
// pub trait ForceModel<S: Storage> {
//     fn apply(&mut self, storage: &mut S, dt: f64);
// }

// // --- ComposedForceModel ---
// // Applies multiple force models in sequence.
// // Each model writes acc — models should *add* to acc, not overwrite.
// // ClearAccModel should be first in the chain to zero acc before forces accumulate.
// pub struct ComposedForceModel<S: Storage> {
//     models: Vec<Box<dyn ForceModel<S>>>,
// }

// impl<S: Storage> ComposedForceModel<S> {
//     pub fn new() -> Self {
//         Self { models: Vec::new() }
//     }

//     pub fn add<FM: ForceModel<S> + 'static>(mut self, model: FM) -> Self {
//         self.models.push(Box::new(model));
//         self
//     }
// }

// impl<S: Storage> ForceModel<S> for ComposedForceModel<S> {
//     fn apply(&mut self, storage: &mut S, dt: f64) {
//         for model in self.models.iter_mut() {
//             model.apply(storage, dt);
//         }
//     }
// }

// // // --- ClearAccModel ---
// // // Zeroes acc on all particles before force accumulation.
// // // Should always be first in a ComposedForceModel chain.
// // pub struct ClearAccModel<const N: usize>;

// // impl<S, const N: usize> ForceModel<S> for ClearAccModel<N>
// // where
// //     S: crate::sim::storage::AosStorage,
// //     S::Item: NewtonianItem<N>,
// // {
// //     fn apply(&mut self, storage: &mut S, _dt: f64) {
// //         for p in storage.iter_mut() {
// //             p.set_acc([0.0; N]);
// //         }
// //     }
// // }

// // // --- NoForceModel ---
// // // No-op — useful for testing or free-flight simulations.
// // pub struct NoForceModel;

// // impl<S: Storage> ForceModel<S> for NoForceModel {
// //     fn apply(&mut self, _storage: &mut S, _dt: f64) {}
// // }

