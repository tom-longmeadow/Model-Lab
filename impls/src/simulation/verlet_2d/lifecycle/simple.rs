use base::sim::{lifecycle::Lifecycle, storage::AosCpuStorage, Bounds};

use crate::simulation::verlet_2d::{particle::Particle, aos_vec_storage::AosVecStorage};


pub struct SimpleLifecycle {
    spawned: bool,
}

impl SimpleLifecycle {
    pub fn new() -> Self {
        Self { spawned: false }
    }
}

impl Lifecycle<AosVecStorage> for SimpleLifecycle {
    fn tick(&mut self, storage: &mut AosVecStorage, _tick: u64, _bounds: &Bounds) {
        if !self.spawned {
            storage.push(Particle::new(150.0, 150.0, 100.0));
            self.spawned = true;
        }
    }
}