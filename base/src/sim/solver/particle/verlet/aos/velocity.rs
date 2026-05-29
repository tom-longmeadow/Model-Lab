// use crate::sim::{solver::Solver, storage::AosStorage};

// // --- Velocity Verlet --- (baseline, weak — kept for comparison)
// // vel = vel + acc * dt
// // pos = pos + vel * dt
// // Non-deterministic if forces depend on other particles.
// pub struct AosVelocityVerletSolver<const N: usize, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     pub force_model: FM,
//     _marker: PhantomData<S>,
// }

// impl<S, const N: usize, FM> Solver<S> for AosVelocityVerletSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
//         self.force_model.apply(storage, dt);
//     }

//     fn substep(&mut self, storage: &mut S, dt: f64) {
//         for p in storage.iter_mut() {
//             let pos = p.pos();
//             let vel = p.vel();
//             let acc = p.acc();
//             p.set_vel(std::array::from_fn(|i| vel[i] + acc[i] * dt));
//             p.set_pos(std::array::from_fn(|i| pos[i] + vel[i] * dt));
//         }
//     }

//     fn post_step(&mut self, _: &mut S, _: f64) {}
// }

// impl<S, const N: usize, FM> ParticleSolver<S> for AosVelocityVerletSolver<N, S, FM>
// where
//     S:  AosStorage,
//     FM: ForceModel<S>,
//     S::Item: NewtonianItem<N>,
// {
//     type Force = FM;
//     fn force_model(&mut self) -> &mut FM { &mut self.force_model }
// }