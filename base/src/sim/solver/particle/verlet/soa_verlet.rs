// use crate::sim::{solver::Solver, storage::{SoaStorage, soa_vec::SoaLayout}};

// // 2D layout — col indices are constants
// pub struct Particle2D; // marker type only, no fields
// impl SoaLayout for Particle2D {
//     // cols: [pos_x, pos_y, vel_x, vel_y, acc_x, acc_y]
//     const STRIDES: &'static [usize] = &[8, 8, 8, 8, 8, 8]; 
// }
// impl Particle2D {
//     pub const POS_X: usize = 0;
//     pub const POS_Y: usize = 1;
//     pub const VEL_X: usize = 2;
//     pub const VEL_Y: usize = 3;
//     pub const ACC_X: usize = 4;
//     pub const ACC_Y: usize = 5;
// }

// // 3D layout — same pattern, 9 cols
// pub struct Particle3D;
// impl Particle3D {
//     pub const POS_X: usize = 0;
//     pub const POS_Y: usize = 1;
//     pub const POS_Z: usize = 2;
//     // vel, acc follow...
// }

// pub struct VerletSolver;

// impl<S, T> Solver<S> for VerletSolver
// where
//     S: SoaStorage<Item = T>,
//     T: VerletLayout,  // trait that provides the col index constants
// {
//     fn substep(&mut self, storage: &mut S, dt: f64) {
//         // Can process each column as a plain slice — SIMD auto-vectorises here
//         let pos_x = storage.col_mut::<f64>(T::POS_X);
//         let vel_x = storage.col_mut::<f64>(T::VEL_X);  // borrow conflict — need split borrows
//         // ...
//     }
    
//     fn pre_step(&mut self, storage: &mut S, dt: f64, tick: u64) {
//         todo!()
//     }
    
//     fn post_step(&mut self, storage: &mut S) {
//         todo!()
//     }
    
//     fn init(&mut self, _storage: &mut S) {}
    
//     fn substep_count(&self) -> usize { 1 }
// }