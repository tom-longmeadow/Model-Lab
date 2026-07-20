use crate::{math::{FloatScalar, Vector, VectorMask}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags}};



pub struct VerletSoaConstraint;
impl VerletSoaConstraint {

    pub fn apply_bounds<V, F>( 
        positions: &mut [V],         
        positions_old: &mut [V],     
        radii: &[V::Scalar],
        dt: V::Scalar,
        environment: &ParticleEnvironment<V, F>,
    ) where 
        V: Vector + 'static,
        V::Scalar: FloatScalar + 'static,
        F: CollisionFlags + 'static,
    {
        let max_len = positions.len();
        
        if max_len == 0 
            || positions_old.len() < max_len
            || radii.len() < max_len 
        {
            return; 
        }

        let positions = &mut positions[..max_len];
        let positions_old = &mut positions_old[..max_len]; 
        let radii = &radii[..max_len];

        let bounds_min = environment.space.bounds.min;
        let bounds_max = environment.space.bounds.max;
        
        let restitution = environment.tuning.physics.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * environment.tuning.physics.friction);
        let base_noise = environment.state.runtime_jitter;

        for i in 0..max_len {
            let p_pos = positions[i];
            let p_pos_old = positions_old[i];
            
            let vel = p_pos - p_pos_old;
            let r = V::splat(radii[i]);
            
            let min_limit = bounds_min + r;
            let max_limit = bounds_max - r;

            let under_min_mask = p_pos.cmplt(min_limit);
            let over_max_mask = p_pos.cmpgt(max_limit);
            let collision_mask = V::mask_or(under_min_mask, over_max_mask);

            if collision_mask.any() {
                // 1. Clamp the current position components to the boundary limits
                let mut new_pos = p_pos;
                new_pos = V::select(under_min_mask, min_limit, new_pos);
                new_pos = V::select(over_max_mask, max_limit, new_pos);

                // 2. Separate normal (bounced) and tangential (friction) velocity paths
                // To match your old system, we invert the entire incoming velocity vector
                let clean_bounced_vel_normal = (-vel) * restitution;
                let jittered_tangential_vel = (vel * friction_diminish) + (base_noise * dt);

                // Choose component-wise behavior across the SIMD lanes
                let new_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);

                // 3. Write back the updated states
                positions[i] = new_pos;
                
                // 💡 THE CRITICAL VERLET CORRECTION: 
                // We project pos_old backwards from the NEW position using the full inverted velocity vector,
                // forcing the next step's (pos - pos_old) calculation to read the true outward bounce momentum!
                positions_old[i] = new_pos - new_vel;
            } else {
                // Open air path: Apply standard air resistance/friction
                let clean_slowed_vel_tangential = vel * friction_diminish;
                positions_old[i] = p_pos - clean_slowed_vel_tangential;
            }
        }
    }

    
}