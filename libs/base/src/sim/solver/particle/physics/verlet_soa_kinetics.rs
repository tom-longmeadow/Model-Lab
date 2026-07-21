use crate::{math::{FloatScalar, Vector}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags}};
 


pub struct VerletSoaKinetics;
impl VerletSoaKinetics{

     #[inline(always)]
    pub fn apply_uniform_acceleration<V, F>(
        accel: &mut [V],      
        environment: &ParticleEnvironment<V, F>, 
    ) where 
        V: Vector,
        F: CollisionFlags + 'static,
    {
        let len = accel.len();
        if len == 0 { return; }

        let gravity = environment.gravity.get(); 
        
        // Explicit upfront windowing to remove bounds-checking overhead
        let acc_slice = unsafe { accel.get_unchecked_mut(0..len) };

        for i in 0..len {
            unsafe {
                let a = acc_slice.get_unchecked_mut(i);
                *a = *a + gravity;
            }
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
        V: Vector,
        F: CollisionFlags + 'static,
    {
        let len = pos.len();
        if len == 0 || pos_old.len() < len || acc.len() < len { 
            return; 
        }

        // SAFE PHYSICS TUNING: Fall back to clean defaults if values are zero
        let damping = environment.tuning.physics.global_damping;
        let damping_factor = if damping > V::Scalar::ZERO {
            (-damping * v_dt).exp()
        } else {
            V::Scalar::ONE
        };

        let max_vel = environment.tuning.physics.max_velocity;
        let max_displacement = if max_vel > V::Scalar::ZERO { max_vel * v_dt } else { V::Scalar::from_f64(1000.0) };
        let max_disp_sqr = max_displacement * max_displacement;
        let dt_sq = v_dt * v_dt;

        let pos_slice = unsafe { pos.get_unchecked_mut(0..len) };
        let old_slice = unsafe { pos_old.get_unchecked_mut(0..len) };
        let acc_slice = unsafe { acc.get_unchecked_mut(0..len) };

        for i in 0..len {
            unsafe {
                let current_pos_ref = pos_slice.get_unchecked_mut(i);
                let old_pos_ref = old_slice.get_unchecked_mut(i);
                let acc_ref = acc_slice.get_unchecked_mut(i);

                let p_curr = *current_pos_ref;
                let p_old = *old_pos_ref;
                let a = *acc_ref;

                // 1. Calculate explicit movement velocity and forces
                let velocity = (p_curr - p_old) * damping_factor;
                let impulse = a * dt_sq;
                let step = velocity + impulse;
                
                // 2. Measure actual scalar step distance
                let dist_sqr = step.length_squared();

                // 3. Safe, non-collapsing Speed Cap
                let final_step = if dist_sqr > max_disp_sqr {
                    let factor = max_displacement / dist_sqr.sqrt();
                    step * factor
                } else {
                    step
                };

                // 4. Pristine Verlet Writes
                *old_pos_ref = p_curr;
                *current_pos_ref = p_curr + final_step;
                *acc_ref = V::ZERO; // Flush acceleration safely
            }
        }
    }
}
        