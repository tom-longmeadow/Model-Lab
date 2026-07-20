pub mod grid; 
pub mod collision_registry;
pub mod grid_key;

use crate::{aabb::AABB, insets::Insets, math::Vector, sim::solver::particle::space::{ grid::UniformGrid, grid_key::GridKey}};
 use std::hash::Hash;


pub struct GridSpace<V> 
where 
    V: Vector 
{
    pub bounds: AABB<V>,
    pub insets: Insets<V>,
    pub grid: UniformGrid<V>,
}

impl<V: Vector> GridSpace<V> 
where
    V::Quantized: Hash + Eq + Copy + GridKey
{
    /// Creates a new `GridSpace` with a specific cell size and default spatial layouts.
    pub fn new(cell_size: V::Scalar) -> Self {
        Self { 
            bounds: AABB::<V>::zero(), 
            insets: Insets::<V>::zero(), 
            grid: UniformGrid::<V>::new(cell_size), // 🟢 This will now compile cleanly!
        }
    }

    #[inline]
    pub fn with_insets(mut self, insets: Insets<V>) -> Self {
        self.insets = insets; 
        self
    }

    #[inline]
    pub fn with_bounds(mut self, bounds: AABB<V>) -> Self { 
        self.bounds = bounds; 
        self
    }
}
 