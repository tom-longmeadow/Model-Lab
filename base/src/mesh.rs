pub mod kind;
pub mod vertex;
pub mod batch;
pub mod ui;
pub mod primitives;

use crate::mesh::kind::*;
use crate::mesh::vertex::*;
pub type Index = u32;


#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub mesh_type: MeshKind,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
}

impl Mesh {
    pub fn new(mesh_type: MeshKind, vertices: Vec<Vertex>, indices: Vec<Index>) -> Self {
        Self {
            mesh_type,
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
}