
struct ScreenUniforms {
    aspect_ratio:  f32,
    screen_width:  f32,
    screen_height: f32,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniforms; 

 

// FIX: Reconstructs a precise f32 out of the f64 raw layout bit channels
fn unpack_f64_to_f32(low_bits: u32, high_bits: u32) -> f32 {
    let sign: f32 = select(1.0, -1.0, (high_bits & 0x80000000u) != 0u);
    let exponent: i32 = i32((high_bits >> 20u) & 0x7FFu) - 1023;
    
    if (exponent < -126) { return 0.0; }
    if (exponent > 127)  { return sign * 3.40282347e+38; } // Precision clip clamp
    
    // Extract the 20 mantissa bits from the high word
    let mantissa_high: u32 = high_bits & 0x0FFFFFu;
    
    // Combine the high and low mantissa words, and scale by the real 52-bit f64 mantissa offset
    // This perfectly restores the native scale, eliminating the 4x reduction bug!
    let exact_mantissa = f32(mantissa_high) * pow(2.0, -20.0) + f32(low_bits) * pow(2.0, -52.0);
    
    return sign * (1.0 + exact_mantissa) * pow(2.0, f32(exponent));
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @location(4) raw_pos:    vec4<f32>, 
    @location(5) raw_radius: vec2<f32>, 
    @location(6) color:      vec4<f32>, 
) -> VertexOutput {

    // 1. Generate the 4 unique quad geometry corners procedurally
    let quad_positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0), // Bottom-Left
        vec2<f32>( 1.0, -1.0), // Bottom-Right
        vec2<f32>( 1.0,  1.0), // Top-Right
        vec2<f32>(-1.0,  1.0)  // Top-Left
    );

    let quad_uvs = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );

    // 2. Unpack the unindexed 0..5 stream into localized quad indices [0, 1, 2, 0, 2, 3]
    let index_map = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let target_quad_idx = index_map[vertex_idx % 6u];

    let generated_position = quad_positions[target_quad_idx];
    let generated_uv       = quad_uvs[target_quad_idx];

    // 3. Process instance attributes (f64 bit shifting)
    let pos_bits_x_low  = bitcast<u32>(raw_pos.x);
    let pos_bits_x_high = bitcast<u32>(raw_pos.y);
    let pos_bits_y_low  = bitcast<u32>(raw_pos.z);
    let pos_bits_y_high = bitcast<u32>(raw_pos.w);

    let radius_bits_low  = bitcast<u32>(raw_radius.x);
    let radius_bits_high = bitcast<u32>(raw_radius.y);

    // Unpack positions and radii using the corrected math
    let pixel_x = unpack_f64_to_f32(pos_bits_x_low, pos_bits_x_high);
    let pixel_y = unpack_f64_to_f32(pos_bits_y_low, pos_bits_y_high);
    let radius  = unpack_f64_to_f32(radius_bits_low, radius_bits_high);

    // Convert pixel coordinates to NDC space
    let ndc_x = (pixel_x / screen.screen_width) * 2.0 - 1.0;
    let ndc_y = (pixel_y / screen.screen_height) * 2.0 - 1.0;
 
    // Multiplying by 2.0 maps the pixel ratio onto the 2.0-unit total span of NDC space.
    // This perfectly aligns the 2-unit-wide base quad with your true physics metrics!
    let radius_ndc_x = (radius / screen.screen_width) * 2.0;
    let radius_ndc_y = (radius / screen.screen_height) * 2.0;

    // Use our procedurally generated position coordinates instead of 'in.position'
    let scaled_offset = vec2<f32>(
        generated_position.x * radius_ndc_x,
        generated_position.y * radius_ndc_y
    );
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x + scaled_offset.x, ndc_y + scaled_offset.y, 0.0, 1.0);
    out.color         = color;
    out.uv            = generated_uv; // Output the exact procedurally generated UV coordinates
    return out;
}
