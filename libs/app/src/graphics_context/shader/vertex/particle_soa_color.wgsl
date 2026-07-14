
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}



// fn vs_particle_soa_color(
//     @location(0) position: vec3<f32>,
//     @location(3) color: vec4<f32>,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4<f32>(position, 1.0);
//     out.color = color;
//     return out;
// }

// fn vs_particle_soa_radius_color(
//     @location(0) position: vec3<f32>,
//     @location(3) color: vec4<f32>,
//     @location(4) radius: f32,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4<f32>(position, 1.0);
//     out.point_size = radius * 2.0;
//     out.color = color;
//     return out;
// }

// fn vs_particle_soa_lit(
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
//     @location(2) uv: vec2<f32>,
//     @location(3) color: vec4<f32>,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4<f32>(position, 1.0);
//     out.color = color;
//     out.normal = normal;
//     out.uv = uv;
//     return out;
// }

// fn vs_particle_soa_radius_lit(
//     @location(0) position: vec3<f32>,
//     @location(1) normal: vec3<f32>,
//     @location(2) uv: vec2<f32>,
//     @location(3) color: vec4<f32>,
//     @location(4) radius: f32,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4<f32>(position, 1.0);
//     out.point_size = radius * 2.0;
//     out.color = color;
//     out.normal = normal;
//     out.uv = uv;
//     return out;
// }