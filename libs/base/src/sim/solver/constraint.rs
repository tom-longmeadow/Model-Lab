use crate::math::{DVec2, DVec3};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Insets {
    pub fn uniform(value: f64) -> Self {
        Self { left: value, top: value, right: value, bottom: value, }
    }
 
    pub fn symmetrical(horizontal: f64, vertical: f64) -> Self {
        Self { left: horizontal, top: vertical, right: horizontal, bottom: vertical, }
    }
 
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self { left, top, right, bottom, }
    }
}
 
/// A lightweight 2D axis-aligned bounding box constraint  
/// Handled as pure spatial bounds—material properties are now cleanly isolated in ParticlePhysicsTuning.
#[derive(Clone, Copy, Debug, Default)]
pub struct RectConstraint {
    pub min: DVec2,
    pub max: DVec2,
}

impl RectConstraint { 

    /// Constructs a direct raw 2D boundary box from anything that can convert into a DVec2.
    /// This accepts glam::DVec2, [f64; 2], (f64, f64), and more.
    pub fn new<V>(min: V, max: V) -> Self
    where
        V: Into<DVec2>,
    {
        Self {
            min: min.into(),
            max: max.into(),
        }
    }

    
    pub fn from_constraint(other: &RectConstraint, insets: &Insets) -> Self {  
        Self::from_dvec2(other.min, other.max, insets)
    }

    pub fn from_dvec3(min: DVec3, max: DVec3, insets: &Insets) -> Self {
        Self::from_dvec2(min.truncate(), max.truncate(), insets)
    }

    #[inline(always)]
    pub fn from_dvec2(min: DVec2, max: DVec2, insets: &Insets) -> Self {
        Self {
            min: DVec2::new(min.x + insets.left, min.y + insets.top),
            max: DVec2::new(max.x - insets.right, max.y - insets.bottom),
        }
    } 
}
