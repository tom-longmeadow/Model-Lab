

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.point_size = model.radius * 2.0;
    out.color = model.color;
    return out;
}