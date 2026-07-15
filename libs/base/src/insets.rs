use crate::math::{FloatScalar, Vector};


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets<V> {
    pub min_offsets: V, // 2D: (left, top)     | 3D: (left, top, near)
    pub max_offsets: V, // 2D: (right, bottom) | 3D: (right, bottom, far)
}

impl<V: Vector> Insets<V> {
     pub fn uniform(value: f64) -> Self {
        // Convert the f64 into whatever float type V expects (f32 or f64)
        let scalar_value = <V::Scalar as FloatScalar>::from_f64(value);
        
        Self { 
            min_offsets: V::splat(scalar_value),
            max_offsets: V::splat(scalar_value),
        }
    }

    pub fn zero() -> Self {
        Self::uniform(0.0)
    }
 
    /// Creates symmetrical insets from a vector of per-axis padding amounts.
    /// - 2D: axis_padding holds (horizontal, vertical)
    /// - 3D: axis_padding holds (horizontal, vertical, depth)
    pub fn symmetrical(axis_padding: V) -> Self {
        Self {
            // Note: Since V implements Copy, you can remove .clone() entirely
            min_offsets: axis_padding,
            max_offsets: axis_padding,
        }
    }

     
}