pub mod kind;
pub mod vertex;
pub mod batch;
pub mod ui;
pub mod primitives;

use crate::math::Vec2;
use crate::math::Vec3;
use crate::math::Vec4;
use crate::mesh::kind::*;
use crate::mesh::vertex::*;
pub type Index = u32;


#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub kind: MeshKind,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
}

impl Mesh {
    pub fn new(mesh_type: MeshKind, vertices: Vec<Vertex>, indices: Vec<Index>) -> Self {
        Self {
            kind: mesh_type,
            vertices,
            indices,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Creates a unit quad mesh centered at the origin spanning from -1.0 to 1.0.
    /// It populates the UV coordinates using the exact [-1.0, 1.0] range expected by your system.
    pub fn unit_quad(kind: MeshKind) -> Self {
        let vertices = vec![
            Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(-1.0, -1.0), Vec4::ONE),
            Vertex::new(Vec3::new( 1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new( 1.0, -1.0), Vec4::ONE),
            Vertex::new(Vec3::new( 1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new( 1.0,  1.0), Vec4::ONE),
            Vertex::new(Vec3::new(-1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(-1.0,  1.0), Vec4::ONE),
        ];

        // Maps the index elements safely to your engine's internal Index type
        let indices = vec![
            0 as Index, 1 as Index, 2 as Index, 
            0 as Index, 2 as Index, 3 as Index
        ];

        Self {
            kind,
            vertices,
            indices,
        }
    }
 
    // pub fn unit_quad_vertices() -> [Vertex; 4] { 
    // [
    //     Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(-1.0, -1.0), Vec4::ONE),
    //     Vertex::new(Vec3::new( 1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new( 1.0, -1.0), Vec4::ONE),
    //     Vertex::new(Vec3::new( 1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new( 1.0,  1.0), Vec4::ONE),
    //     Vertex::new(Vec3::new(-1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(-1.0,  1.0), Vec4::ONE),
    // ]


//     pub fn unit_quad_indices() -> [u16; 6] {
//         [0, 1, 2, 0, 2, 3]
//     }
}
 