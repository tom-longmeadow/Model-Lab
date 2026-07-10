use base::sim::storage::{AosCpuStorage, CpuStorage, Storage};

use crate::simulation::verlet_2d::particle::Particle;


pub struct AosVecStorage {
    particles: Vec<Particle>,
}

impl AosVecStorage {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }
}

impl Storage for AosVecStorage {
    fn len(&self) -> usize { 
        self.particles.len() 
    }
    
    fn capacity(&self) -> usize { 
        self.particles.capacity() 
    }
    
    fn clear(&mut self) { 
        self.particles.clear(); 
    }
    
    fn remove_indices(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in indices {
            self.particles.swap_remove(i);
        }
    }
}

impl CpuStorage for AosVecStorage {
    fn new(capacity: usize) -> Self {
        Self { particles: Vec::with_capacity(capacity) }
    }
}

impl AosCpuStorage for AosVecStorage {
    type Item = Particle;

    fn push(&mut self, item: Particle) {
        self.particles.push(item);
    }

    fn swap_remove(&mut self, index: usize) -> Particle {
        self.particles.swap_remove(index)
    }

    fn as_slice(&self) -> &[Particle] {
        &self.particles
    }

    fn as_slice_mut(&mut self) -> &mut [Particle] {
        &mut self.particles
    }
}