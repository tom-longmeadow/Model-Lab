//! Math types and operations.
//! 
//! This module re-exports types from `glam` with type aliases.
//! To swap out the math library, only this file needs to change.

// Re-export the entire glam crate for advanced usage
use glam;

// Core vector types - these are the "public API" of your math module
pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;

pub type DVec2 = glam::DVec2;  // f64 versions
pub type DVec3 = glam::DVec3;
pub type DVec4 = glam::DVec4;

pub type IVec2 = glam::IVec2;  // i32 versions
pub type IVec3 = glam::IVec3;
pub type IVec4 = glam::IVec4;

pub type UVec2 = glam::UVec2;  // u32 versions
pub type UVec3 = glam::UVec3;
pub type UVec4 = glam::UVec4;

// Matrix types
pub type Mat2 = glam::Mat2;
pub type Mat3 = glam::Mat3;
pub type Mat4 = glam::Mat4;

pub type DMat2 = glam::DMat2;
pub type DMat3 = glam::DMat3;
pub type DMat4 = glam::DMat4;

// Quaternions
pub type Quat = glam::Quat;
pub type DQuat = glam::DQuat;

// Affine transforms (if you need them)
pub type Affine2 = glam::Affine2;
pub type Affine3A = glam::Affine3A;

// ---------------------------------------------------------------------------
// Helper Traits
// ---------------------------------------------------------------------------

/// A trait for vector types that provides compile-time dimension info.
pub trait Vector {
    /// The number of components in the vector (e.g., 2 for Vec2, 3 for Vec3).
    const DIM: usize;
}

impl Vector for Vec2 { const DIM: usize = 2; }
impl Vector for Vec3 { const DIM: usize = 3; }
impl Vector for Vec4 { const DIM: usize = 4; }

impl Vector for DVec2 { const DIM: usize = 2; }
impl Vector for DVec3 { const DIM: usize = 3; }
impl Vector for DVec4 { const DIM: usize = 4; }

// Utility functions (optional - add domain-specific helpers here)
/// Linear interpolation between two vectors
#[inline]
pub fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t)
}

/// Clamp a vector's length to a maximum value
#[inline]
pub fn clamp_length(v: Vec3, max_length: f32) -> Vec3 {
    if v.length_squared() > max_length * max_length {
        v.normalize() * max_length
    } else {
        v
    }
}


#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn test_clamp_length() {
        let v = Vec3::new(3.0, 4.0, 0.0); // length = 5
        let clamped = clamp_length(v, 3.0);
        assert!((clamped.length() - 3.0).abs() < 0.001);
    }
}