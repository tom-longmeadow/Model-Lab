//! Math types and operations.
//! 
//! This module re-exports types from `glam` with type aliases.
//! To swap out the math library, only this file needs to change.

// Re-export the entire glam crate for advanced usage
use glam;
pub use glam::FloatExt; 
pub const EPSILON: f32 = 1e-5;

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


/// Minimal float operations needed for generic simulation math.
pub trait FloatScalar:
    Copy
    + PartialOrd
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    fn sqrt(self) -> Self;
    fn from_f64(v: f64) -> Self;
}

impl FloatScalar for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    #[inline] fn sqrt(self) -> Self { f32::sqrt(self) }
    #[inline] fn from_f64(v: f64) -> Self { v as f32 }
}

impl FloatScalar for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    #[inline] fn sqrt(self) -> Self { f64::sqrt(self) }
    #[inline] fn from_f64(v: f64) -> Self { v }
}

pub trait Vector: Copy {
    const DIM: usize;
    type Scalar;
    fn dot(self, other: Self) -> Self::Scalar;
}

impl Vector for glam::Vec2 {
    const DIM: usize = 2;
    type Scalar = f32;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}

impl Vector for glam::Vec3 {
    const DIM: usize = 3;
    type Scalar = f32;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}

impl Vector for glam::Vec4 {
    const DIM: usize = 4;
    type Scalar = f32;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}

impl Vector for glam::DVec2 {
    const DIM: usize = 2;
    type Scalar = f64;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}

impl Vector for glam::DVec3 {
    const DIM: usize = 3;
    type Scalar = f64;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}

impl Vector for glam::DVec4 {
    const DIM: usize = 4;
    type Scalar = f64;
    #[inline]
    fn dot(self, other: Self) -> Self::Scalar {
        self.dot(other)
    }
}
 
 
/// Spatial bounds defining a rectangular region in 3D space. 
#[derive(Debug, Clone, Copy, Default)]
pub struct Bounds {
    pub min: DVec3,
    pub max: DVec3,
}

impl Bounds {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    /// Creates a 2D bounding box resting flat on the Z = 0 plane.
    pub fn new_2d<V>(min_2d: V, max_2d: V) -> Self
    where
        V: Into<DVec2>,
    {
        let m1 = min_2d.into();
        let m2 = max_2d.into();

        Self {
            // Extends the 2D vectors into 3D vectors by appending Z = 0.0
            min: DVec3::new(m1.x, m1.y, 0.0),
            max: DVec3::new(m2.x, m2.y, 0.0),
        }
    }

    /// Finds the exact center point of your 3D bounds
    pub fn center(&self) -> DVec3 {
        // Leverages built-in midpoint math on the vectors directly!
        self.min.midpoint(self.max)
    }

    /// Gets the size (width, height, depth) as a vector
    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    /// Checks if a particle is inside your 3D bounds
    pub fn contains(&self, point: DVec3) -> bool {
        // glam gives you component-wise comparisons natively
        point.cmple(self.max).all() && point.cmpge(self.min).all()
    }

    /// Automatically expands the bounding box to encapsulate a new particle position
    pub fn encapsulate(&mut self, point: DVec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Returns a new `Bounds` shrunk uniformly by a scalar border on all sides.
    #[must_use] // Warns the developer if they forget to assign the returned value
    pub fn shrink(&self, border: f64) -> Self {
        // Move min coordinates UP, move max coordinates DOWN
        let offset = DVec3::splat(border);
        Self {
            min: self.min + offset,
            max: self.max - offset,
        }
    }

    /// Returns a new `Bounds` shrunk by distinct x, y, and z border amounts.
    #[must_use]
    pub fn shrink_axes<V>(&self, border: V) -> Self 
    where 
        V: Into<DVec3> 
    {
        // Using Into allows passing a tuple (x, y, z) or a DVec3 seamlessly
        let offset = border.into();
        Self {
            min: self.min + offset,
            max: self.max - offset,
        }
    }

}