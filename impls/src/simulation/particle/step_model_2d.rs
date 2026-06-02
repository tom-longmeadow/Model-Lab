use base::sim::{
    solver::{
        StepModel,
        step_model::{ClearAcc, ConstantAccel},
    },
    solver::verlet::step_model::VerletDimConstraint,
    storage::{AosStorage, aos_vec::AosVecStorage, soa_vec::SoaVecStorage},
    storage::verlet::{aos::AosVerletItem, soa::SoaVerletStorage},
};
use super::particle_2d::VerletParticle2d;

/// Gravity + axis-aligned box wall constraint for 2D Verlet particle sims.
///
/// - `pre`  — clear acc, apply constant downward gravity on y.
/// - `post` — clamp each particle inside [x_min..x_max] × [y_min..y_max]
///             with velocity reflection.
///
/// One type, two [`StepModel`] impls — AoS and SoA share identical fields
/// and the same scalar kernels; only the storage iteration differs.
pub struct BoxModel2d {
    pub gravity: ConstantAccel,
    pub x_wall:  VerletDimConstraint,
    pub y_wall:  VerletDimConstraint,
}

impl BoxModel2d {
    /// - `gravity`     — downward acceleration (positive = down, e.g. `9.81`)
    /// - `x_min/x_max` — horizontal walls
    /// - `y_min/y_max` — vertical walls
    /// - `restitution` — `1.0` = elastic, `0.0` = inelastic
    pub fn new(
        gravity:     f64,
        x_min: f64, x_max: f64,
        y_min: f64, y_max: f64,
        restitution: f64,
    ) -> Self {
        Self {
            gravity: ConstantAccel::new(-gravity), // negate: gravity acts in -y
            x_wall:  VerletDimConstraint::new(x_min, x_max, restitution),
            y_wall:  VerletDimConstraint::new(y_min, y_max, restitution),
        }
    }
}

// ---------------------------------------------------------------------------
// AoS impl — iterates over particle structs
// ---------------------------------------------------------------------------

impl StepModel<AosVecStorage<VerletParticle2d>> for BoxModel2d {
    fn pre(&mut self, storage: &mut AosVecStorage<VerletParticle2d>, _dt: f64) {
        for p in storage.iter_mut() {
            let acc = p.acc_mut();
            ClearAcc::apply(&mut acc[0]);
            ClearAcc::apply(&mut acc[1]);
            self.gravity.apply(&mut acc[1]);  // gravity on y
        }
    }

    fn post(&mut self, storage: &mut AosVecStorage<VerletParticle2d>, _dt: f64) {
        for p in storage.iter_mut() {
            let (pos, pos_old, _) = p.pos_pos_old_mut_acc();
            self.x_wall.apply(&mut pos[0], &mut pos_old[0]);
            self.y_wall.apply(&mut pos[1], &mut pos_old[1]);
        }
    }
}

// ---------------------------------------------------------------------------
// SoA impl — iterates over contiguous f64 columns (cache-hot)
// ---------------------------------------------------------------------------

impl StepModel<SoaVecStorage<VerletParticle2d>> for BoxModel2d {
    fn pre(&mut self, storage: &mut SoaVecStorage<VerletParticle2d>, _dt: f64) {
        for a in storage.acc_col_mut(0) { ClearAcc::apply(a); }
        for a in storage.acc_col_mut(1) {
            ClearAcc::apply(a);
            self.gravity.apply(a);
        }
    }

    fn post(&mut self, storage: &mut SoaVecStorage<VerletParticle2d>, _dt: f64) {
        let (px, px_old, _) = storage.pos_pos_old_col_mut_acc(0);
        for i in 0..px.len() { self.x_wall.apply(&mut px[i], &mut px_old[i]); }

        let (py, py_old, _) = storage.pos_pos_old_col_mut_acc(1);
        for i in 0..py.len() { self.y_wall.apply(&mut py[i], &mut py_old[i]); }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use base::sim::{
        solver::{Solver},
        solver::verlet::solver::{AosVerletSolver, SoaVerletSolver},
        storage::Storage,
    };
    use super::super::particle_2d::{AosStorage2d, SoaStorage2d};

    const GRAVITY:    f64 = 9.81;
    const RESTITUTION:f64 = 1.0;  // perfectly elastic for predictable tests
    const BOX_SIZE:   f64 = 10.0;
    const DT:         f64 = 0.01;
    const STEPS:      usize = 500;

    fn aos_sim() -> (AosVerletSolver<AosStorage2d, BoxModel2d, 2>, AosStorage2d) {
        let model = BoxModel2d::new(GRAVITY, 0.0, BOX_SIZE, 0.0, BOX_SIZE, RESTITUTION);
        let solver = AosVerletSolver::new(model);
        let mut storage = AosStorage2d::new(64);
        storage.push(VerletParticle2d {
            pos:     [5.0, 5.0],
            pos_old: [5.0, 5.0],
            acc:     [0.0, 0.0],
        });
        (solver, storage)
    }

    fn soa_sim() -> (SoaVerletSolver<SoaStorage2d, BoxModel2d, 2>, SoaStorage2d) {
        let model = BoxModel2d::new(GRAVITY, 0.0, BOX_SIZE, 0.0, BOX_SIZE, RESTITUTION);
        let solver = SoaVerletSolver::new(model);
        let mut storage = SoaStorage2d::new(64);
        storage.push(VerletParticle2d {
            pos:     [5.0, 5.0],
            pos_old: [5.0, 5.0],
            acc:     [0.0, 0.0],
        });
        (solver, storage)
    }

    #[test]
    fn aos_particle_stays_in_box() {
        let (mut solver, mut storage) = aos_sim();
        for tick in 0..STEPS {
            solver.pre_step(&mut storage, DT, tick as u64);
            solver.sub_step(&mut storage, DT);
            solver.post_step(&mut storage, DT);
        }
        let p = storage.get(0);
        assert!(p.pos[0] >= 0.0 && p.pos[0] <= BOX_SIZE, "x out of bounds: {}", p.pos[0]);
        assert!(p.pos[1] >= 0.0 && p.pos[1] <= BOX_SIZE, "y out of bounds: {}", p.pos[1]);
    }

    #[test]
    fn soa_particle_stays_in_box() {
        use base::sim::storage::verlet::soa::SoaVerletStorage;
        let (mut solver, mut storage) = soa_sim();
        for tick in 0..STEPS {
            solver.pre_step(&mut storage, DT, tick as u64);
            solver.sub_step(&mut storage, DT);
            solver.post_step(&mut storage, DT);
        }
        let x = storage.pos_col(0)[0];
        let y = storage.pos_col(1)[0];
        assert!(x >= 0.0 && x <= BOX_SIZE, "x out of bounds: {x}");
        assert!(y >= 0.0 && y <= BOX_SIZE, "y out of bounds: {y}");
    }

    #[test]
    fn aos_and_soa_produce_identical_trajectories() {
        let (mut aos_solver, mut aos_storage) = aos_sim();
        let (mut soa_solver, mut soa_storage) = soa_sim();

        for tick in 0..STEPS {
            aos_solver.pre_step(&mut aos_storage, DT, tick as u64);
            aos_solver.sub_step(&mut aos_storage, DT);
            aos_solver.post_step(&mut aos_storage, DT);

            soa_solver.pre_step(&mut soa_storage, DT, tick as u64);
            soa_solver.sub_step(&mut soa_storage, DT);
            soa_solver.post_step(&mut soa_storage, DT);
        }

        use base::sim::storage::verlet::soa::SoaVerletStorage;
        let aos_p = aos_storage.get(0);
        let soa_x = soa_storage.pos_col(0)[0];
        let soa_y = soa_storage.pos_col(1)[0];

        assert!((aos_p.pos[0] - soa_x).abs() < 1e-12, "x diverged: {} vs {}", aos_p.pos[0], soa_x);
        assert!((aos_p.pos[1] - soa_y).abs() < 1e-12, "y diverged: {} vs {}", aos_p.pos[1], soa_y);
    }
}
