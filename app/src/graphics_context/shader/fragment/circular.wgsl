

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(in.uv, vec2<f32>(0.5, 0.5));
    // Soft edge over the outer ~10% of the radius for anti-aliasing.
    let alpha = 1.0 - smoothstep(0.45, 0.5, dist);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}