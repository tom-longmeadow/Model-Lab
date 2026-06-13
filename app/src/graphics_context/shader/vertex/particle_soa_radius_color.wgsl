

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(3) color: vec4<f32>,
    @location(4) radius: f32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);
    out.point_size = radius * 2.0;
    out.color = color;
    return out;
}

 