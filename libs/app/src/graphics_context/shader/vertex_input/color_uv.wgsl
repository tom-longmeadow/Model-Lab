struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32, 
    @location(0) particle_pos: vec3<f32>,
    @location(1) particle_color: vec4<f32>,
    @location(2) particle_scale: f32,
};
