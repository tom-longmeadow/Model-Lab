struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @builtin(point_size) point_size: f32,   
    @location(0) color: vec4<f32>,
};