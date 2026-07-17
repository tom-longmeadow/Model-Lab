use std::mem;
use base::mesh::vertex::Vertex;

/// An extension trait to attach wgpu capabilities to your core Vertex struct
/// without pulling wgpu into your core mathematics crate!
pub trait WgpuVertexExt {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

impl WgpuVertexExt for Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        // Since Vertex is repr(C), vertex_attr_array maps directly to Vec3, Vec2, and Vec4
        const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x3, // maps perfectly to position: Vec3
            1 => Float32x3, // maps perfectly to normal: Vec3
            2 => Float32x2, // maps perfectly to uv: Vec2
            3 => Float32x4, // maps perfectly to color: Vec4
        ];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

// If any other systems still need a traditional 4-vertex quad array, 
// you can generate it using your core math types directly:


//     pub fn unit_quad_indices() -> [u16; 6] {
//         [0, 1, 2, 0, 2, 3]
//     }



//use std::mem;




// #[repr(C)]
// #[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct GpuVertex {
//     pub position: [f32; 3],
//     pub normal:   [f32; 3],
//     pub uv:       [f32; 2],
//     pub color:    [f32; 4],
// }

// impl GpuVertex {
//     pub fn layout() -> wgpu::VertexBufferLayout<'static> {
//         const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
//             0 => Float32x3, 
//             1 => Float32x3, 
//             2 => Float32x2, 
//             3 => Float32x4, 
//         ];

//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &ATTRIBS,
//         }
//     }

//     pub fn unit_quad() -> [Self; 4] {
//     [
//         Self { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
//         Self { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0, -1.0], color: [1.0; 4] },
//         Self { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
//         Self { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0,  1.0], color: [1.0; 4] },
//     ]
// }

//     pub fn unit_quad_indices() -> [u16; 6] {
//         [0, 1, 2, 0, 2, 3]
//     }
// }

// pub fn unit_quad() ->[GpuVertex]{
//         let unit_quad_vertices = [
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0, -1.0], color: [1.0; 4] },
//             GpuVertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [ 1.0,  1.0], color: [1.0; 4] },
//             GpuVertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [-1.0,  1.0], color: [1.0; 4] },
//         ];
//     }