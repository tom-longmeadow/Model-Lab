// use std::marker::PhantomData;
// use crate::sim::{
//     solver::{Solver, 
//         particle::{ClassicVerletItem, ForceModel, ParticleSolver}
//     }, 
//     storage::AosStorage
// };
 

// // --- Classic Verlet --- (deterministic, no stored velocity)
// // pos_new = 2*pos - pos_old + acc * dt²
// // pos_old = pos
// // Naturally deterministic — all reads from snapshot, writes to new pos.
// pub struct AosClassicVerletSolver<const N: usize, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: ClassicVerletItem<N>,
// {
//     pub force_model: FM,
//     _marker: PhantomData<S>,
// }

// impl<S, const N: usize, FM> Solver<S> for AosClassicVerletSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: ClassicVerletItem<N>,
// {
//     fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
//         self.force_model.apply(storage, dt);
//     }

//     fn substep(&mut self, storage: &mut S, dt: f64) {
//         let dt2 = dt * dt;
//         for p in storage.iter_mut() {
//             let pos     = p.pos();
//             let pos_old = p.pos_old();
//             let acc     = p.acc();
//             let pos_new = std::array::from_fn(|i| 2.0 * pos[i] - pos_old[i] + acc[i] * dt2);
//             p.set_pos_old(pos);
//             p.set_pos(pos_new);
//         }
//     }

//     fn post_step(&mut self, _: &mut S, _: f64) {}
// }

// impl<S, const N: usize, FM> ParticleSolver<S> for AosClassicVerletSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: ClassicVerletItem<N>,
// {
//     type Force = FM;
//     fn force_model(&mut self) -> &mut FM { &mut self.force_model }
// }