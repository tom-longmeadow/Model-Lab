

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate the distance from the exact middle of the quad.
    // Because your UV coordinates range from -1.0 to 1.0, 
    // the mathematical center is exactly at (0.0, 0.0).
    let dist = length(in.uv);
    
    // 2. Smoothly blend the outer edge for perfect anti-aliasing.
    // This scales down opacity *only* between the radius of 0.95 and 1.0.
    let alpha = 1.0 - smoothstep(0.95, 1.0, dist);
    
    // 3. Discard any pixel that falls completely outside the circle.
    if (dist > 1.0) {
        discard;
    }
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}


// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     // 1. Calculate distance from the true center of the quad (0.0, 0.0)
//     let dist = distance(in.uv, vec2<f32>(0.0, 0.0));
    
//     // 2. Apply anti-aliasing smoothly over the outer 5% edge up to a radius of 1.0
//     let alpha = 1.0 - smoothstep(0.95, 1.0, dist);
    
//     // 3. Discard unrendered pixels immediately
//     if (alpha <= 0.0) {
//         discard;
//     }
    
//     return vec4<f32>(in.color.rgb, in.color.a * alpha);
// }