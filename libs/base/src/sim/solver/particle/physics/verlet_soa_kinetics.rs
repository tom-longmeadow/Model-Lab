use crate::{math::{FloatScalar, Vector}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags}};
use std::hash::Hash;


pub struct VerletSoaKinetics;
impl VerletSoaKinetics{

     #[inline(always)]
    pub fn apply_uniform_acceleration<V, F>(
        accel: &mut [V],      
        environment: &ParticleEnvironment<V, F>, 
    ) where 
        V: Vector + 'static,
        V::Scalar: FloatScalar, 
        V::Quantized: Hash + Eq + Copy, 
        F: CollisionFlags + 'static, // Keeps environment bounds aligned
    {
        let gravity = environment.gravity.get(); 
        // 🟢 SAFE & AUTO-VECTORIZED: Iterators eliminate bounds checking entirely
        for a in accel.iter_mut() {
            *a = *a + gravity; 
        }
    }

    #[inline(always)]
    pub fn update_kinetics<V, F>(
        pos: &mut [V],
        pos_old: &mut [V],
        acc: &mut [V],
        v_dt: V::Scalar,       
        environment: &ParticleEnvironment<V, F>,
    ) where 
        V: Vector + 'static,
        V::Scalar: FloatScalar, 
        V::Quantized: Hash + Eq + Copy, 
        F: CollisionFlags + 'static,
    {
        let len = pos.len();
        if len == 0 || pos_old.len() < len || acc.len() < len { 
            return; 
        }

        let damping_factor = (-environment.tuning.physics.global_damping * v_dt).exp(); 
        let dt_sq = v_dt * v_dt;

        let positions = &mut pos[..len];
        let positions_old = &mut pos_old[..len];
        let accelerations = &mut acc[..len];

        let iter = positions.iter_mut()
            .zip(positions_old.iter_mut())
            .zip(accelerations.iter_mut());

        for ((current_pos, old_pos), acc_val) in iter {
            let p_curr = *current_pos;
            let p_old = *old_pos;
            let a = *acc_val;

            let displacement = p_curr - p_old;
            // 🟢 dt_sq applies perfectly here to the pure raw acceleration
            let next_pos = p_curr + (displacement * damping_factor) + (a * dt_sq);

            *old_pos = p_curr;
            *current_pos = next_pos;
            *acc_val = V::ZERO; // Reset buffer for next frame's forces
        }
    }

}