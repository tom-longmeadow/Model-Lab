
use crate::mesh::{kind::MeshKind, vertex::Vertex, Mesh};

#[derive(Clone, Debug)]
pub struct MeshBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub kind: MeshKind,
}

impl MeshBatch {
    pub fn new(kind: MeshKind) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            kind,
        }
    }

    /// Append a mesh into this batch, offsetting its indices
    pub fn push(&mut self, mesh: &Mesh) {
        assert_eq!(
            self.kind, mesh.mesh_type,
            "cannot batch meshes of different kinds"
        );

        let offset = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&mesh.vertices);
        self.indices.extend(mesh.indices.iter().map(|i| i + offset));
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn into_mesh(self) -> Mesh {
        Mesh::new(self.kind, self.vertices, self.indices)
    }
}

/// Collects meshes and sorts them into batches by MeshKind
#[derive(Clone, Debug)]
pub struct MeshBatcher {
    pub triangles: MeshBatch,
    pub lines: MeshBatch,
    pub points: MeshBatch,
}

impl MeshBatcher {
    pub fn new() -> Self {
        Self {
            triangles: MeshBatch::new(MeshKind::Triangle),
            lines: MeshBatch::new(MeshKind::Line),
            points: MeshBatch::new(MeshKind::Point),
        }
    }

    pub fn push(&mut self, mesh: &Mesh) {
        match mesh.mesh_type {
            MeshKind::Triangle => self.triangles.push(mesh),
            MeshKind::Line => self.lines.push(mesh),
            MeshKind::Point => self.points.push(mesh),
        }
    }

    /// Returns only non-empty batches as meshes
    pub fn finish(self) -> Vec<Mesh> {
        let mut out = Vec::new();
        if !self.triangles.is_empty() {
            out.push(self.triangles.into_mesh());
        }
        if !self.lines.is_empty() {
            out.push(self.lines.into_mesh());
        }
        if !self.points.is_empty() {
            out.push(self.points.into_mesh());
        }
        out
    }
}