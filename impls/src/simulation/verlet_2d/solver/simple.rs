use base::sim::{solver::{Solver, integrator::Verlet}, storage::AosCpuStorage};

use crate::simulation::verlet_2d::vec_storage::VecStorage;

pub struct SimpleSolver;

impl Solver<VecStorage> for SimpleSolver {
    fn sub_step(&mut self, storage: &mut VecStorage, dt: f64) {
        for p in storage.iter_mut() {
            for i in 0..2 {
                Verlet::step(&mut p.pos[i], &mut p.pos_old[i], p.acc[i], dt);
            }
        }
    }
}