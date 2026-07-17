use wgpu::util::DeviceExt;

pub trait UniformLayout: bytemuck::Pod + bytemuck::Zeroable {
    const LABEL: &'static str;

    /// Automatically generates the correct layout for this specific uniform type
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(Self::LABEL),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX, // Or VERTEX | FRAGMENT if needed
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None, // Can use NonZeroU64::new(mem::size_of::<Self>() as u64) for extra safety
                },
                count: None,
            }],
        })
    }
}

/// A self-contained, ergonomic GPU uniform container.
pub struct Uniform<T: UniformLayout> {
    pub data: T,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<T: UniformLayout> Uniform<T> {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, data: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(T::LABEL),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(T::LABEL),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { data, buffer, bind_group }
    }
 
    pub fn upload(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }
}

// use wgpu::util::DeviceExt;

 
// pub trait BindableUniform: bytemuck::Pod + bytemuck::Zeroable {
//     const LABEL: &'static str;
 
//     fn create_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some(Self::LABEL),
//             contents: bytemuck::bytes_of(self),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         })
//     }
 
//     fn update_buffer(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
//         queue.write_buffer(buffer, 0, bytemuck::bytes_of(self));
//     }
// }

//  pub struct UniformComponent<T: BindableUniform> {
//     pub data: T,
//     pub buffer: wgpu::Buffer,
//     pub bind_group: wgpu::BindGroup,
// }

// impl<T: BindableUniform> UniformComponent<T> {
//     pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, data: T) -> Self {
//         let buffer = data.create_buffer(device);

//         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some(T::LABEL),
//             layout,
//             entries: &[wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: buffer.as_entire_binding(),
//             }],
//         });

//         Self { data, buffer, bind_group }
//     }
 
//     pub fn update(&self, queue: &wgpu::Queue) {
//         self.data.update_buffer(queue, &self.buffer);
//     }
// }