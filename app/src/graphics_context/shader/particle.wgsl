// This struct must match the GpuVertex struct in Rust.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // Note: This shader assumes positions are already in clip space (-1.0 to 1.0).
    // If your particle positions are in world space, you'll need to multiply
    // by a view-projection matrix here.
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Use the built-in point_coord to make the square point a circle.
    // point_coord gives a 2D coordinate from (0.0, 0.0) to (1.0, 1.0)
    // within the point primitive.
    let dist = distance(in.point_coord, vec2<f32>(0.5, 0.5));

    // Discard pixels outside the circle's radius.
    if (dist > 0.5) {
        discard;
    }

    // Optional: Add a soft edge for anti-aliasing
    // let alpha = 1.0 - smoothstep(0.45, 0.5, dist);

    return vec4<f32>(in.color.rgb, in.color.a);
    // To use the soft edge, use this line instead:
    // return vec4<f32>(in.color.rgb, in.color.a * alpha);
}