use crate::{math::{FloatScalar, Vector}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags}};



pub struct VerletSoaConstraint;
impl VerletSoaConstraint {

     #[inline(always)]
    pub fn apply_bounds<V, F>( 
        positions: &mut [V],         
        positions_old: &mut [V],     
        radii: &[V::Scalar],
        dt: V::Scalar,
        environment: &ParticleEnvironment<V, F>,
    ) where 
        V: Vector,
        F: CollisionFlags + 'static,
    {
        let max_len = positions.len();
        if max_len == 0 
            || positions_old.len() < max_len
            || radii.len() < max_len 
        {
            return; 
        }

        // FIXED: Swapped out standard slicing for unchecked windowing 
        // to stay perfectly aligned with your high-speed update loops
        let pos_slice = unsafe { positions.get_unchecked_mut(0..max_len) };
        let pos_old_slice = unsafe { positions_old.get_unchecked_mut(0..max_len) }; 
        let radii_slice = unsafe { radii.get_unchecked(0..max_len) };

        let bounds_min = environment.space.bounds.min;
        let bounds_max = environment.space.bounds.max;
        
        let restitution = environment.tuning.physics.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * environment.tuning.physics.friction);
        
        // Hoisted jitter vector expression to register space upfront
        let base_noise_vec =  environment.state.runtime_jitter;
        let jitter_term = base_noise_vec * dt;

        for i in 0..max_len {
            unsafe {
                let current_pos_ptr = pos_slice.get_unchecked_mut(i);
                let old_pos_ptr = pos_old_slice.get_unchecked_mut(i);

                let p_pos = *current_pos_ptr;
                let p_pos_old = *old_pos_ptr;
                
                let vel = p_pos - p_pos_old;
                let r = V::splat(*radii_slice.get_unchecked(i));
                
                let min_limit = bounds_min + r;
                let max_limit = bounds_max - r;

                let under_min_mask = p_pos.cmplt(min_limit);
                let over_max_mask = p_pos.cmpgt(max_limit);
                let collision_mask = V::mask_or(under_min_mask, over_max_mask);

                // FIXED: 100% Branchless Position Clamping via pure SIMD select masks
                let mut new_pos = p_pos;
                new_pos = V::select(under_min_mask, min_limit, new_pos);
                new_pos = V::select(over_max_mask, max_limit, new_pos);

                // FIXED: Compute both execution paths simultaneously without CPU branches.
                // Modern CPUs can execute these arithmetic operations in parallel registers faster 
                // than they can recover from a single branch misprediction!
                
                // Path A: The Wall Collision Bounce
                let clean_bounced_vel_normal = (-vel) * restitution;
                let jittered_tangential_vel = (vel * friction_diminish) + jitter_term;
                let collision_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);
                let collision_pos_old = new_pos - collision_vel;

                // Path B: Open Air Flight Paths
                let open_air_vel = vel * friction_diminish;
                let open_air_pos_old = p_pos - open_air_vel;

                // FIXED: Use a single unified select line to assign final outputs 
                // across the hardware SIMD lanes based on lane collision states.
                *current_pos_ptr = new_pos;
                *old_pos_ptr = V::select(collision_mask, collision_pos_old, open_air_pos_old);
            }
        }
    }

    
}