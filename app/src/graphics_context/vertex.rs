


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub uv:       [f32; 2],
    pub color:    [f32; 4],
}

impl From<base::mesh::vertex::Vertex> for GpuVertex {
    fn from(v: base::mesh::vertex::Vertex) -> Self {
        Self {
            position: v.position,
            normal:   v.normal,
            uv:       v.uv,
            color:    v.color,
        }
    }
}

impl GpuVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GpuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0,                                          shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32;3]>() as u64,           shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32;6]>() as u64,           shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: mem::size_of::<[f32;8]>() as u64,           shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}