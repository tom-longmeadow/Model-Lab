use base::mesh::kind::MeshKind;

pub struct GpuMeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub kind: MeshKind,
}