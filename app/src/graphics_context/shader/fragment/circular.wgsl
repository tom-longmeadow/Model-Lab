

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(in.point_coord, vec2<f32>(0.5, 0.5));
    if (dist > 0.5) {
        discard;
    }
    return in.color;
}