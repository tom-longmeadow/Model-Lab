use crate::{math::{FloatScalar, Vector}, 
sim::solver::particle::{ 
    environment::ParticleEnvironment, flags::CollisionFlags, space::grid_key::GridKey}, ui::layout::color::Color};
use std::hash::Hash;



pub struct VerletSoaPrestep;
impl VerletSoaPrestep{

    
    #[inline(always)]
    pub fn update_color_from_velocity<V, F>(
        min_speed: V::Scalar,
        max_speed: V::Scalar,
        pos: &[V],            
        pos_old: &[V],        
        color: &mut [Color],         
        v_dt: V::Scalar, 
        _tick: u64,
        environment: &ParticleEnvironment<V, F>, 
    ) where 
        V: Vector + 'static,
        V::Scalar: FloatScalar, 
        V::Quantized: Hash + Eq + Copy,
        F: CollisionFlags + 'static, // Aligns completely with your environment strategy
    {
        // 🟢 BASE LENGTH: Deriving the primary loop target size from the output array
        let len = color.len();
        if len == 0 || pos.len() < len || pos_old.len() < len {
            return;
        }

        let zero = V::Scalar::ZERO;
        let one = V::Scalar::ONE;
        let speed_range = max_speed - min_speed; 

        // 🟢 SLICE WINDOWING: Up-front sizing removes all internal panic checks
        let color = &mut color[..len];
        let positions = &pos[..len];
        let positions_old = &pos_old[..len];

        for i in 0..len {
            let p = positions[i];
            let p_old = positions_old[i];
            
            let velocity = p - p_old;
            let speed = velocity.length() / v_dt;
            
            let diff = speed - min_speed;
            let adjusted_speed = if diff > zero { diff } else { zero };
            
            let raw_percentage = adjusted_speed / speed_range;
            let percentage = if raw_percentage < zero {
                zero
            } else if raw_percentage > one {
                one
            } else {
                raw_percentage
            }; 
            
            let percent_f64 = percentage.to_f64(); 
            let c = environment.state.get_color(percent_f64);
            
            color[i] = c; 
        }
    }
 

    #[inline(always)]
    pub fn update_grid_cell_size<V, F>(  
        radii: &[V::Scalar],  
        environment: &mut ParticleEnvironment<V, F>,
    ) where 
        V: Vector + 'static,
        V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static,
    {
         
        type S<V> = <V as Vector>::Scalar; 
        let mut min_radius = S::<V>::INFINITY;
        let mut max_radius = S::<V>::NEG_INFINITY;

        for &r in radii.iter() {
            if r < min_radius { min_radius = r; }
            if r > max_radius { max_radius = r; }
        }

        let max_diameter = max_radius + max_radius;
        environment.space.grid.set_cell_size(max_diameter);  
    }

    #[inline(always)] 
    pub fn update_jitter<V, F>( 
        tick: u64,
        environment: &mut ParticleEnvironment<V, F>,
    ) where 
        V: Vector + 'static,
        V::Quantized: Hash + Eq + Copy,
        F: CollisionFlags + 'static,
    {
        environment.state.update_jitter(tick);
    }
}