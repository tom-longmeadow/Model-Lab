use base::sim::{solver::{Solver, integrator::Verlet, verlet::RectConstraint}, storage::AosCpuStorage, Bounds};

use crate::simulation::verlet_2d::aos_vec_storage::AosVecStorage;

pub struct GravitySolver{
    bounds: RectConstraint,
    gravity: f64,
}

impl GravitySolver
{
    pub fn new(sim_bounds: Bounds, restitution: f64, gravity: f64) -> Self {
        Self { 
            bounds: RectConstraint::new(
                sim_bounds.x_min, 
                sim_bounds.x_max, 
                sim_bounds.y_min, 
                sim_bounds.y_max, 
                restitution
            ),
            gravity
        }
    }
}
 

impl Solver<AosVecStorage> for GravitySolver {
    
    fn pre_step(&mut self, _storage: &mut AosVecStorage, _dt: f64, _tick: u64, bounds: &Bounds) {
        // Update constraints from current bounds (supports dynamic resize)
        self.bounds = RectConstraint::new(
            bounds.x_min,
            bounds.x_max,
            bounds.y_min,
            bounds.y_max,
            self.bounds.restitution,
        );
    }

    fn sub_step(&mut self, storage: &mut AosVecStorage, dt: f64) {
        for p in storage.iter_mut() {

            p.acc.y = -self.gravity;

            Verlet::step(&mut p.pos.x, &mut p.pos_old.x, p.acc.x, dt);
            Verlet::step(&mut p.pos.y, &mut p.pos_old.y, p.acc.y, dt);

            self.bounds.apply(&mut p.pos.x, &mut p.pos_old.x, &mut p.pos.y, &mut p.pos_old.y);
        }
    }
}