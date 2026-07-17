use base::mesh::{Mesh, kind::MeshKind};
use wgpu::util::DeviceExt; 


pub struct GpuMeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer:  wgpu::Buffer,
    pub index_count:   u32,
    pub kind:          MeshKind,
}

impl GpuMeshBuffers {
    /// Creates GPU buffers directly out of your backend-agnostic core Mesh structure
    pub fn from_mesh(device: &wgpu::Device, mesh: &Mesh) -> Self {

        // Safe direct bitcast thanks to #[repr(C)] + bytemuck on your core Vertex struct
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Safe direct bitcast of your internal engine Index slice (u16 or u32)
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.index_count() as u32,
            kind: mesh.kind,
        }
    } 
}