
use crate::math::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: Vec4,
}

impl Vertex {
    pub fn new(
        position: Vec3,
        normal: Vec3,
        uv: Vec2,
        color: Vec4,
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            color,
        }
    }

    
    pub fn from_arrays(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
        color: [f32; 4],
    ) -> Self {
        Self {
            position: position.into(),
            normal: normal.into(),
            uv: uv.into(),
            color: color.into(),
        }
    }

}