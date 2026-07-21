use crate::{math::{FloatScalar, Vector}, 
sim::solver::particle::{ 
    environment::ParticleEnvironment, flags::CollisionFlags, space::grid_key::GridKey}, ui::layout::color::Color};
use std::hash::Hash;



pub struct VerletSoaPrestep;
impl VerletSoaPrestep{
    
    #[inline(always)]
    pub fn update_color_from_velocity<V,F>(
        min_speed: V::Scalar,
        max_speed: V::Scalar,
        pos: &[V],            
        pos_old: &[V],        
        color: &mut [Color],         
        v_dt: V::Scalar, 
        _tick: u64,
        environment: &ParticleEnvironment<V, F>, 
    ) where 
        V:Vector,
        F: CollisionFlags + 'static,  
    {
        let len = color.len();
        if len == 0 || pos.len() < len || pos_old.len() < len {
            return;
        }

        let zero = V::Scalar::ZERO;
        let one = V::Scalar::ONE;
        let speed_range = max_speed - min_speed; 

        // FIXED: Upfront sub-slicing using exact bounds variables 
        // to match your high-speed kinetics engine strategy.
        let colors = unsafe { color.get_unchecked_mut(0..len) };
        let positions = unsafe { pos.get_unchecked(0..len) };
        let positions_old = unsafe { pos_old.get_unchecked(0..len) };

        // Optimization Strategy: Extract state metadata once upfront!
        // This ensures the loop body never jumps out of line to chase pointers.
        let state = &environment.state;

        for i in 0..len {
            unsafe {
                let p = *positions.get_unchecked(i);
                let p_old = *positions_old.get_unchecked(i);
                
                let velocity = p - p_old;
                let speed = velocity.length() / v_dt;
                
                let diff = speed - min_speed;
                // Branchless Clamping: Transforms cleanly into native CPU hardware cmov/blend lines
                let adjusted_speed = if diff > zero { diff } else { zero };
                
                let raw_percentage = adjusted_speed / speed_range;
                let percentage = if raw_percentage < zero {
                    zero
                } else if raw_percentage > one {
                    one
                } else {
                    raw_percentage
                }; 
                
                // PERFORMANCE CRITICAL NOTE: Ensure environment.state.get_color() 
                // is explicitly marked with #[inline(always)] in your code!
                // If it isn't, replace this call with raw math calculation loops here.
                let percent_f64 = percentage.to_f64(); 
                *colors.get_unchecked_mut(i) = state.get_color(percent_f64);
            }
        }
    }

    #[inline(always)]
    pub fn update_grid_cell_size<V,F>(  
        radii: &[V::Scalar],  
        environment: &mut ParticleEnvironment<V, F>,
    ) where
        V: Vector,
        V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static,
    {
        let len = radii.len();
        if len == 0 { return; }

        let mut min_radius = V::Scalar::INFINITY;
        let mut max_radius = V::Scalar::NEG_INFINITY;

        let slice = unsafe { radii.get_unchecked(0..len) };

        // FIXED: Branchless reduction stream.
        // By avoiding nested 'if' tracking structures, LLVM can pack 
        // this loop into highly efficient SIMD comparison instructions.
        for i in 0..len {
            unsafe {
                let r = *slice.get_unchecked(i);
                min_radius = if r < min_radius { r } else { min_radius };
                max_radius = if r > max_radius { r } else { max_radius };
            }
        }

        let max_diameter = max_radius + max_radius;
        environment.space.grid.set_cell_size(max_diameter);  
    }

    #[inline(always)] 
    pub fn update_jitter<V,F>( 
        tick: u64,
        environment: &mut ParticleEnvironment<V, F>,
    ) where 
        V:Vector,
        F: CollisionFlags + 'static, // FIXED: Removed the stray trailing parenthesis here
    {
        environment.state.update_jitter(tick);
    }
}