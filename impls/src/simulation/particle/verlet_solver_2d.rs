use base::sim::solver::verlet::solver::{AosVerletSolver, SoaVerletSolver};
use super::particle_2d::{AosStorage2d, SoaStorage2d};
use super::step_model_2d::BoxModel2d;

/// AoS Verlet solver for 2D particles with a [`BoxModel2d`] step model.
pub type AosVerletSolver2d = AosVerletSolver<AosStorage2d, BoxModel2d, 2>;

/// SoA Verlet solver for 2D particles with a [`BoxModel2d`] step model.
pub type SoaVerletSolver2d = SoaVerletSolver<SoaStorage2d, BoxModel2d, 2>;
