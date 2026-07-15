//use crate::{math::Vector, sim::{solver::particle::verlet_particle::VerletParticle, storage::{AosCpuStorage, CpuStorage, Storage}}};

use crate::sim::solver::{aos_vec_storage::AosVecStorage, particle::verlet_particle::VerletParticle};

 
 
pub type VerletParticleAosVecStorage<V> = AosVecStorage<VerletParticle<V>>;

 