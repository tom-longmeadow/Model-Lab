// use std::rc::Rc;
// use std::cell::RefCell;
// use std::marker::PhantomData;
// use crate::sim::{
//     bounds::Bounds,
//     creator::Creator,
//     solver::particle::{NewtonianParticle, Particle, VerletParticle},
//     storage::{AosStorage, SoaNewtonianStorage, SoaVerletStorage, SoaStorage},
// };

// // ---------------------------------------------------------------------------
// // NewtonianVolumeCreator
// // ---------------------------------------------------------------------------

// /// Fills the bounds to `fill_ratio` of its volume, one particle per tick.
// /// When bounds shrink, removes the slowest (lowest |vel|²) particles first.
// /// When bounds grow, spawns new particles at the centre via `spawn()`.
// ///
// /// - `B`  — any [`Bounds<N>`] (shared with [`BoundsConstraint`] via [`SharedBounds`])
// /// - `N`  — spatial dimension
// /// - `F`  — `Fn() -> S::Item` factory for new particles
// ///
// /// ```ignore
// /// let bounds  = shared(Rect::new([0.0; 2], [10.0; 2]));
// /// let creator = NewtonianVolumeCreator::new(Rc::clone(&bounds), 0.8, 1.0, || Particle2d::default());
// /// ```
// pub struct NewtonianVolumeCreator<B, const N: usize, F> {
//     bounds:       Rc<RefCell<B>>,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     spawn:        F,
//     _n:           PhantomData<fn() -> [f64; N]>,
// }

// impl<B, const N: usize, F> NewtonianVolumeCreator<B, N, F> {
//     /// - `fill_ratio`   — target fraction of volume to fill, e.g. `0.8`
//     /// - `particle_vol` — volume of a single particle (e.g. `π r²` for a disc)
//     pub fn new(
//         bounds:       Rc<RefCell<B>>,
//         fill_ratio:   f64,
//         particle_vol: f64,
//         spawn:        F,
//     ) -> Self {
//         Self { bounds, fill_ratio, particle_vol, spawn, _n: PhantomData }
//     }
// }

// impl<S, B, const N: usize, F> Creator<S> for NewtonianVolumeCreator<B, N, F>
// where
//     S: AosStorage,
//     S::Item: NewtonianParticle<N>,
//     B: Bounds<N>,
//     F: Fn() -> S::Item,
// {
//     fn tick(&mut self, storage: &mut S, _tick: u64) {
//         let target = {
//             let b = self.bounds.borrow();
//             ((b.volume() * self.fill_ratio) / self.particle_vol).floor() as usize
//         };

//         // grow — one particle per tick to avoid frame spikes
//         if storage.len() < target {
//             storage.push((self.spawn)());
//             return;
//         }

//         // shrink — remove the particle with the lowest |vel|² (most at-rest)
//         if storage.len() > target {
//             let slowest = storage
//                 .iter()
//                 .enumerate()
//                 .min_by(|(_, a), (_, b)| {
//                     let va: f64 = (*a).vel().iter().map(|v| v * v).sum();
//                     let vb: f64 = (*b).vel().iter().map(|v| v * v).sum();
//                     va.partial_cmp(&vb).unwrap()
//                 })
//                 .map(|(i, _)| i);
//             if let Some(i) = slowest { storage.swap_remove(i); }
//         }
//     }
// }

// // ---------------------------------------------------------------------------
// // VerletVolumeCreator
// // ---------------------------------------------------------------------------

// /// Same volume-filling logic for Verlet particles.
// /// Verlet has no explicit velocity — rest is detected via displacement magnitude:
// /// `|pos - pos_old|` is proportional to speed × dt.
// /// Particles with the smallest displacement are removed first when shrinking.
// ///
// /// - `B`  — any [`Bounds<N>`]
// /// - `N`  — spatial dimension
// /// - `F`  — `Fn() -> S::Item` factory returning a particle with `pos == pos_old`
// pub struct VerletVolumeCreator<B, const N: usize, F> {
//     bounds:       Rc<RefCell<B>>,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     spawn:        F,
//     _n:           PhantomData<fn() -> [f64; N]>,
// }

// impl<B, const N: usize, F> VerletVolumeCreator<B, N, F> {
//     /// - `fill_ratio`   — target fraction of volume to fill, e.g. `0.8`
//     /// - `particle_vol` — volume of a single particle
//     pub fn new(
//         bounds:       Rc<RefCell<B>>,
//         fill_ratio:   f64,
//         particle_vol: f64,
//         spawn:        F,
//     ) -> Self {
//         Self { bounds, fill_ratio, particle_vol, spawn, _n: PhantomData }
//     }
// }

// impl<S, B, const N: usize, F> Creator<S> for VerletVolumeCreator<B, N, F>
// where
//     S: AosStorage,
//     S::Item: VerletParticle<N>,
//     B: Bounds<N>,
//     F: Fn() -> S::Item,
// {
//     fn tick(&mut self, storage: &mut S, _tick: u64) {
//         let target = {
//             let b = self.bounds.borrow();
//             ((b.volume() * self.fill_ratio) / self.particle_vol).floor() as usize
//         };

//         if storage.len() < target {
//             storage.push((self.spawn)());
//             return;
//         }

//         // shrink — remove particle with smallest |pos - pos_old|² (least movement)
//         if storage.len() > target {
//             let stilled = storage
//                 .iter()
//                 .enumerate()
//                 .min_by(|(_, a), (_, b)| {
//                     let da: f64 = (0..N).map(|i| ((*a).pos()[i] - (*a).pos_old()[i]).powi(2)).sum();
//                     let db: f64 = (0..N).map(|i| ((*b).pos()[i] - (*b).pos_old()[i]).powi(2)).sum();
//                     da.partial_cmp(&db).unwrap()
//                 })
//                 .map(|(i, _)| i);
//             if let Some(i) = stilled { storage.swap_remove(i); }
//         }
//     }
// }

// // ---------------------------------------------------------------------------
// // SoaNewtonianVolumeCreator
// // ---------------------------------------------------------------------------

// /// SoA-native volume creator for Newtonian particles.
// /// Scans the raw `vel` column directly instead of going through the
// /// [`NewtonianParticle`] trait — a single contiguous pass, fully cache-hot.
// ///
// /// Performance advantage over [`NewtonianVolumeCreator`] (AoS) during shrink:
// /// the velocity column `[x0,x1,...,y0,y1,...]` is one flat array vs striding
// /// through AoS structs where velocity may be buried behind position and mass.
// ///
// /// Use this with [`SoaSolver`] to get the full SoA benefit end-to-end.
// pub struct SoaNewtonianVolumeCreator<B, const N: usize, F> {
//     bounds:       Rc<RefCell<B>>,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     spawn:        F,
//     _n:           PhantomData<fn() -> [f64; N]>,
// }
// impl<B, const N: usize, F> SoaNewtonianVolumeCreator<B, N, F> {
//     pub fn new(
//         bounds:       Rc<RefCell<B>>,
//         fill_ratio:   f64,
//         particle_vol: f64,
//         spawn:        F,
//     ) -> Self {
//         Self { bounds, fill_ratio, particle_vol, spawn, _n: PhantomData }
//     }
// }
// impl<S, B, const N: usize, F> Creator<S> for SoaNewtonianVolumeCreator<B, N, F>
// where
//     S: SoaStorage + SoaNewtonianStorage,
//     B: Bounds<N>,
//     F: Fn() -> S::Item,
// {
//     fn tick(&mut self, storage: &mut S, _tick: u64) {
//         let target = {
//             let b = self.bounds.borrow();
//             ((b.volume() * self.fill_ratio) / self.particle_vol).floor() as usize
//         };

//         if storage.len() < target {
//             storage.push((self.spawn)());
//             return;
//         }

//         // shrink — scan the flat vel column: component c of particle i is vel[i + c*len].
//         // All N components for one particle are stride `len` apart — but we sum
//         // across components per particle, so we walk particle-major over the column.
//         // Still a single contiguous array vs chasing AoS struct pointers.
//         if storage.len() > target {
//             let len = storage.len();
//             let vel = storage.vel(); // &[f64], blocked layout
//             let slowest = (0..len)
//                 .min_by(|&i, &j| {
//                     let vi: f64 = (0..N).map(|c| vel[i + c * len].powi(2)).sum();
//                     let vj: f64 = (0..N).map(|c| vel[j + c * len].powi(2)).sum();
//                     vi.partial_cmp(&vj).unwrap()
//                 });
//             if let Some(i) = slowest { storage.swap_remove(i); }
//         }
//     }
// }

// // ---------------------------------------------------------------------------
// // SoaVerletVolumeCreator
// // ---------------------------------------------------------------------------

// /// SoA-native volume creator for Verlet particles.
// /// Scans `pos` and `pos_old` columns directly — displacement per particle
// /// is `|pos - pos_old|²`, computed from two flat column arrays.
// ///
// /// Same cache advantage as [`SoaNewtonianVolumeCreator`] vs the AoS version.
// pub struct SoaVerletVolumeCreator<B, const N: usize, F> {
//     bounds:       Rc<RefCell<B>>,
//     fill_ratio:   f64,
//     particle_vol: f64,
//     spawn:        F,
//     _n:           PhantomData<fn() -> [f64; N]>,
// }
// impl<B, const N: usize, F> SoaVerletVolumeCreator<B, N, F> {
//     pub fn new(
//         bounds:       Rc<RefCell<B>>,
//         fill_ratio:   f64,
//         particle_vol: f64,
//         spawn:        F,
//     ) -> Self {
//         Self { bounds, fill_ratio, particle_vol, spawn, _n: PhantomData }
//     }
// }
// impl<S, B, const N: usize, F> Creator<S> for SoaVerletVolumeCreator<B, N, F>
// where
//     S: SoaStorage + SoaVerletStorage,
//     B: Bounds<N>,
//     F: Fn() -> S::Item,
// {
//     fn tick(&mut self, storage: &mut S, _tick: u64) {
//         let target = {
//             let b = self.bounds.borrow();
//             ((b.volume() * self.fill_ratio) / self.particle_vol).floor() as usize
//         };

//         if storage.len() < target {
//             storage.push((self.spawn)());
//             return;
//         }

//         if storage.len() > target {
//             let len     = storage.len();
//             let pos     = storage.pos();
//             let pos_old = storage.pos_old();
//             let stilled = (0..len)
//                 .min_by(|&i, &j| {
//                     let di: f64 = (0..N).map(|c| (pos[i + c*len] - pos_old[i + c*len]).powi(2)).sum();
//                     let dj: f64 = (0..N).map(|c| (pos[j + c*len] - pos_old[j + c*len]).powi(2)).sum();
//                     di.partial_cmp(&dj).unwrap()
//                 });
//             if let Some(i) = stilled { storage.swap_remove(i); }
//         }
//     }
// }
