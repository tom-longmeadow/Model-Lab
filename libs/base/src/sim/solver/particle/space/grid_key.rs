 use std::hash::Hash; 
use crate::math::{IVec2, IVec3}; 
 
pub trait GridKey: Hash + Eq + Copy + std::ops::Add<Output = Self> + 'static { 
    const OFFSETS: &'static [Self];
}

impl GridKey for IVec2 {
    const OFFSETS: &'static [Self] = &[
        IVec2::new(1, 0),  // Right
        IVec2::new(1, 1),  // Up-Right
        IVec2::new(0, 1),  // Up
        IVec2::new(-1, 1), // Up-Left
    ];
}

impl GridKey for IVec3 {
    // 13 offsets representing exactly half of a 3x3x3 cube neighborhood.
    // This ensures no 3D cell pair is ever evaluated backward or duplicated.
    const OFFSETS: &'static [Self] = &[
        // Z = 0 plane (current layer, same as 2D)
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 1, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(1, 1, 0),
        
        // Z = 1 plane (the layer directly above/forward)
        IVec3::new(-1, -1, 1),
        IVec3::new(0, -1, 1),
        IVec3::new(1, -1, 1),
        IVec3::new(-1, 0, 1),
        IVec3::new(0, 0, 1),
        IVec3::new(1, 0, 1),
        IVec3::new(-1, 1, 1),
        IVec3::new(0, 1, 1),
        IVec3::new(1, 1, 1),
    ];
}