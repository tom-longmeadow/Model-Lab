
// Instanced particle rendering:
// - Vertex buffer (location 0-3): unit quad vertices (position, normal, uv, color)
// - Instance buffer (location 4-7): per-particle data (already in NDC space)

@vertex
fn vs_main(
    in: VertexInput,                           // Unit quad vertex (locations 0-3)
    @location(4) ndc_position: vec3<f32>,      // Per-particle position in NDC (already transformed)
    @location(5) radius_x: f32,                // Per-particle X radius in NDC units
    @location(6) radius_y: f32,                // Per-particle Y radius in NDC units
    @location(7) color: vec4<f32>,             // Per-particle RGBA color
) -> VertexOutput {
    // Scale unit quad by particle radii (different in X and Y to maintain circular appearance)
    let scaled_pos = vec3<f32>(
        in.position.x * radius_x,
        in.position.y * radius_y,
        in.position.z
    );
    
    // Translate to particle position (already in NDC)
    let final_pos = vec3<f32>(
        scaled_pos.x + ndc_position.x,
        scaled_pos.y + ndc_position.y,
        scaled_pos.z + ndc_position.z
    );
    
    var out: VertexOutput;
    out.position = vec4<f32>(final_pos, 1.0);
    out.color = color;  // Use instance color, not vertex color
    out.uv = in.uv;
    return out;
}

