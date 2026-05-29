// use crate::sim::{solver::Solver, storage::AosStorage};

// // --- Leapfrog (Störmer–Verlet) --- (deterministic, symplectic, best energy conservation)
// // pre_step  : half-kick  vel += acc * 0.5 * dt
// // substep   : drift      pos += vel * dt
// // post_step : recompute acc (via force_fn), then half-kick vel += acc * 0.5 * dt
// //
// // force_fn is called in post_step to recompute acc from new positions.
// // All reads happen before any writes — deterministic and parallelisable.
// pub struct AosLeapfrogSolver<const N: usize, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     pub force_model: FM,
//     _marker: PhantomData<S>,
// }

// impl<S, const N: usize, FM> AosLeapfrogSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     pub fn new(force_model: FM) -> Self {
//         Self { force_model, _marker: PhantomData }
//     }
// }

// impl<S, const N: usize, FM> Solver<S> for AosLeapfrogSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     /// Half-kick — vel += acc * 0.5 * dt
//     fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
//         let half_dt = dt * 0.5;
//         for p in storage.iter_mut() {
//             let vel = p.vel();
//             let acc = p.acc();
//             p.set_vel(std::array::from_fn(|i| vel[i] + acc[i] * half_dt));
//         }
//     }

//     /// Drift — pos += vel * dt
//     fn substep(&mut self, storage: &mut S, dt: f64) {
//         for p in storage.iter_mut() {
//             let pos = p.pos();
//             let vel = p.vel();
//             p.set_pos(std::array::from_fn(|i| pos[i] + vel[i] * dt));
//         }
//     }

//     /// Recompute acc from new positions via force_model, then second half-kick.
//     fn post_step(&mut self, storage: &mut S, dt: f64) {
//         // recompute acc from new positions — force_model is responsible for clearing acc first
//         self.force_model.apply(storage, dt);
//         // second half-kick with updated acc
//         let half_dt = dt * 0.5;
//         for p in storage.iter_mut() {
//             let vel = p.vel();
//             let acc = p.acc();
//             p.set_vel(std::array::from_fn(|i| vel[i] + acc[i] * half_dt));
//         }
//     }
// }

// impl<S, const N: usize, FM> ParticleSolver<S> for AosLeapfrogSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     type Force = FM;
//     fn force_model(&mut self) -> &mut FM { &mut self.force_model }
// }