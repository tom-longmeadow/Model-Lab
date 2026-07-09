use base::sim::{lifecycle::Lifecycle, storage::AosCpuStorage};

use crate::simulation::verlet_2d::{particle::Particle, vec_storage::VecStorage};


pub struct SimpleLifecycle {
    spawned: bool,
}

impl SimpleLifecycle {
    pub fn new() -> Self {
        Self { spawned: false }
    }
}

impl Lifecycle<VecStorage> for SimpleLifecycle {
    fn tick(&mut self, storage: &mut VecStorage, _tick: u64) {
        if !self.spawned {
            storage.push(Particle::default());
            self.spawned = true;
        }
    }
}