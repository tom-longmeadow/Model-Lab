use crate::math::Vector;

pub trait ParticleDataLayout<V: Vector> {
    fn len(&self) -> usize;
    fn radii(&self) -> &[V::Scalar];
    
    // REPLACED positions_mut and old_positions_mut with this unified call
    fn positions_and_old_mut(&mut self) -> (&mut [V], &mut [V]);
    
    // Keep this separate as it doesn't overlap temporally with the method above
    fn positions_mut(&mut self) -> &mut [V];

    fn commit_kinetics(&mut self, max_vel_squared: V::Scalar, sub_step_max_vel: V::Scalar);
}



 