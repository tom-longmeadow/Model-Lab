use crate::{math::{FloatScalar, Vector}, sim::{solver::{aos_vec_storage::AosVecStorage, particle::{data_layout::ParticleDataLayout, verlet_particle::VerletParticle}}, storage::{AosCpuStorage, Storage}}};

 
pub type VerletParticleAosVecStorage<V> = AosVecStorage<VerletParticle<V>>;

pub struct AoSSimulationView<'a, V: Vector> {
    pub storage: &'a mut VerletParticleAosVecStorage<V>,
    pub scratch_pos: &'a mut Vec<V>,
    pub scratch_old: &'a mut Vec<V>,
    pub scratch_radii: &'a [V::Scalar],
}

impl<'a, V: Vector + 'static> ParticleDataLayout<V> for AoSSimulationView<'a, V> {
    #[inline(always)]
    fn len(&self) -> usize { 
        Storage::len(self.storage) 
    }

    #[inline(always)]
    fn radii(&self) -> &[V::Scalar] { 
        self.scratch_radii 
    }

    #[inline(always)]
    fn positions_mut(&mut self) -> &mut [V] { 
        self.scratch_pos 
    }

    #[inline(always)]
    fn positions_and_old_mut(&mut self) -> (&mut [V], &mut [V]) {
        // Safe because scratch_pos and scratch_old are independent unique mutable fields
        (self.scratch_pos, self.scratch_old)
    }

    fn commit_kinetics(&mut self, max_vel_squared: V::Scalar, sub_step_max_vel: V::Scalar) {
        let particle_slice = self.storage.as_slice_mut();
        for (i, p) in particle_slice.iter_mut().enumerate() {
            let pos = self.scratch_pos[i];
            let mut pos_old = self.scratch_old[i];
            let vel = pos - pos_old;
            let vel_sq = vel.length_squared();
            if vel_sq > max_vel_squared {
                pos_old = pos - (vel * (sub_step_max_vel / vel_sq.sqrt()));
            }
            p.pos = pos;
            p.pos_old = pos_old;
        }
    }
}