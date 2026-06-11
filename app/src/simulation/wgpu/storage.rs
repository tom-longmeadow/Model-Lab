// use base::sim::storage::GpuStorage;
// use bytemuck::Pod;
// use wgpu;

// /// A trait for GPU storage specifically implemented with the wgpu API.
// /// This storage is expected to be double-buffered ("ping-pong") to allow
// /// a compute pass to write to one buffer while a render pass reads from the other
// /// within the same frame, avoiding race conditions.
// pub trait WgpuStorage: GpuStorage {
//     /// Creates new storage on the GPU. This requires the wgpu::Device.
//     fn new(device: &wgpu::Device, capacity: usize) -> Self;

//     /// Returns the buffer that should be used for **reading** in the current frame.
//     /// This is the buffer containing the results of the *previous* frame's simulation step.
//     /// A render pass would use this for its vertex data.
//     fn read_buffer(&self) -> &wgpu::Buffer;

//     /// Returns the buffer that should be used for **writing** in the current frame.
//     /// A compute solver would write its new results into this buffer.
//     fn write_buffer(&self) -> &wgpu::Buffer;

//     /// Swaps the read and write buffers. This should be called once per frame,
//     /// after all simulation and rendering commands for the frame have been submitted.
//     fn swap_buffers(&mut self);
// }

// /// GPU-based Struct-of-Arrays storage.
// pub trait SoaWgpuStorage: WgpuStorage {
//     /// Returns the byte offset and size of a specific column within the buffer.
//     /// This layout is assumed to be identical for both read and write buffers.
//     fn column_layout(&self, column_index: usize) -> (wgpu::BufferAddress, wgpu::BufferSize);

//     /// Records a command to upload data from a CPU slice to a specific column
//     /// in the **write** buffer. This is for initializing or injecting data.
//     fn upload_col<T: Pod>(
//         &self,
//         encoder: &mut wgpu::CommandEncoder,
//         column_index: usize,
//         data: &[T],
//     );
// }

// /// GPU-based Array-of-Structs storage.
// pub trait AosWgpuStorage: WgpuStorage {
//     type Item: Pod;

//     /// Records a command to upload data from a CPU slice of items to the
//     /// **write** buffer.
//     fn upload(&self, encoder: &mut wgpu::CommandEncoder, data: &[Self::Item]);
// }



// // // A concrete implementation of a GPU-based storage
// // pub struct SoaGpuParticleStorage {
// //     buffer: wgpu::Buffer,
// //     len: usize,
// //     capacity: usize,
// // }

// // // Implement all the traits...
// // impl base::sim::storage::Storage for SoaGpuParticleStorage { /* ... */ }
// // impl base::sim::storage::GpuStorage for SoaGpuParticleStorage { /* ... */ }
// // impl WgpuStorage for SoaGpuParticleStorage { /* ... */ }

// // // You could even have a GpuSolver that uses compute shaders
// // pub struct GpuVerletSolver { /* ... */ }