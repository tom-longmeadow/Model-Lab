 

pub mod grid;
pub mod collision;


// pub trait Partition<V: Vector>
// where
//     V::Scalar: FloatScalar,
// {
//     fn resize(&mut self, bounds: &Bounds, radii: &[V::Scalar]);
//     fn clear(&mut self);
//     fn populate(&mut self, positions: &[V]);
//     fn find_collisions(&self, positions: &[V], radii: &[V::Scalar], registry: &mut CollisionRegistry<V>);
// }