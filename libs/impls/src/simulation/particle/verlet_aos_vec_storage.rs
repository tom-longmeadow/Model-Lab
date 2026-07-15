use base::{math::Vector, sim::{solver::particle::verlet_particle::VerletParticle, storage::{AosCpuStorage, CpuStorage, Storage}}};
 


pub struct AosVecStorage<V: Vector> {
    particles: Vec<VerletParticle<V>>,
}

impl<V: Vector> AosVecStorage<V> {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }
}

impl<V: Vector> Storage for AosVecStorage<V> {
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

impl<V: Vector> CpuStorage for AosVecStorage<V> {
    fn new(capacity: usize) -> Self {
        Self { particles: Vec::with_capacity(capacity) }
    }
}

impl<V: Vector> AosCpuStorage for AosVecStorage<V> {
    type Item = VerletParticle<V>;

    fn push(&mut self, item: Self::Item) {
        self.particles.push(item);
    }

    fn swap_remove(&mut self, index: usize) -> Self::Item {
        self.particles.swap_remove(index)
    }

    fn as_slice(&self) -> &[Self::Item] {
        &self.particles
    }

    fn as_slice_mut(&mut self) -> &mut [Self::Item] {
        &mut self.particles
    }
}