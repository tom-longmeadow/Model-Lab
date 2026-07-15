use crate::{insets::Insets, math::{FloatScalar, Vector}};


#[derive(Clone, Copy, Debug)]
pub struct AABB<V> {
    pub min: V,
    pub max: V,
}

impl<V: Vector> Default for AABB<V> {
    #[inline]
    fn default() -> Self {
        Self {
            min: V::ZERO,
            max: V::ZERO,
        }
    }
}

impl<V> AABB<V> 
where
    V: Vector,
{
    pub fn new(min: V, max: V) -> Self {
        Self { min, max }
    }

    pub fn from_insets(other: &Self, insets: &Insets<V>) -> Self {
        Self {
            // Since V is Copy, we can add/subtract vectors directly
            min: other.min + insets.min_offsets,
            max: other.max - insets.max_offsets,
        }
    }

    pub fn center(&self) -> V { 
        // Replaces the missing midpoint method using generic scalar math
        let half = <V::Scalar as FloatScalar>::from_f64(0.5);
        (self.min + self.max) * half
    }
 
    pub fn size(&self) -> V {
        self.max - self.min
    } 

    pub fn contains(&self, point: V) -> bool { 
        // Delegated to your updated Vector trait method
        point.contains_point(self.min, self.max)
    }
    
    pub fn encapsulate(&mut self, point: V) {
        // Delegated to your Vector trait min/max components
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Returns a new bounding box shrunk uniformly on all axes by a single scalar amount.
    #[must_use]
    pub fn shrink_uniform(&self, amount: f64) -> Self { 
        let scalar_amount = <V::Scalar as FloatScalar>::from_f64(amount);
        let offset = V::splat(scalar_amount);
        
        Self {
            min: self.min + offset,
            max: self.max - offset,
        }
    }

    /// Returns a new AABB shrunk by distinct per-axis border amounts.
    #[must_use]
    pub fn shrink_axes(&self, border: V) -> Self { 
        Self {
            min: self.min + border,
            max: self.max - border,
        }
    }
}

 
 