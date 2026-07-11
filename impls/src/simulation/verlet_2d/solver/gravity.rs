use base::{math::{Bounds, DVec2}, sim::{solver::{Solver, integrator::Verlet, verlet::RectConstraint}, storage::AosCpuStorage}};

use crate::simulation::verlet_2d::aos_vec_storage::AosVecStorage;

pub struct GravitySolver{
    bounds: RectConstraint,
    gravity: f64,
    inset: f64,
    restitution: f64,
}

impl GravitySolver
{
    pub fn new(bounds: &Bounds, restitution: f64, gravity: f64, inset: f64) -> Self {
        Self { 
            bounds: RectConstraint::from_bounds_with_inset(bounds, inset, restitution),
            gravity,
            inset,
            restitution
        }
    }
}
 

impl Solver<AosVecStorage> for GravitySolver {
    
    fn pre_step(&mut self, _storage: &mut AosVecStorage, _dt: f64, _tick: u64, bounds: &Bounds) {
        self.bounds = RectConstraint::from_bounds_with_inset(bounds,  self.inset, self.restitution);
        //self.bounds = RectConstraint::new(bounds.min.truncate(), bounds.max.truncate(), self.restitution)
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