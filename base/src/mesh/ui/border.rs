use std::f32::consts::PI;

use crate::{
    mesh::{Mesh, kind::MeshKind, vertex::Vertex},
    ui::layout::{border::{BorderKind, BorderStyle}, color::Color, corner::CornerStyle, rect::Rect},
};


pub fn border(rect: Rect, style: BorderStyle, corner: CornerStyle) -> Option<Mesh> {
    match style.kind {
        BorderKind::Solid => Some(border_solid(rect, style, corner)),
        BorderKind::Line  => Some(border_line(rect, style.color, corner)),
        BorderKind::None  => None,
    }
}
 
pub fn border_line(rect: Rect, color: Color, corner: CornerStyle) -> Mesh {
    let [r, g, b, a] = color.as_f32_array();
    let color_arr = [r, g, b, a];

    let vertices = perimeter_vertices(rect, corner, color_arr);
    let n = vertices.len() as u32;

     
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..n {
        indices.push(i);
        indices.push((i + 1) % n);
    }

    Mesh::new(MeshKind::Line, vertices, indices)
}

 
pub fn border_solid(rect: Rect, style: BorderStyle, corner: CornerStyle) -> Mesh {
    let [r, g, b, a] = style.color.as_f32_array();
    let color_arr = [r, g, b, a];

    let width = style.width;

    // outer perimeter
    let outer_verts = perimeter_vertices(rect, corner, color_arr);

    // inner rect shrunk by border width
    let inner_rect = Rect {
        x: rect.x + width,
        y: rect.y + width,
        w: (rect.w - width * 2.0).max(0.0),
        h: (rect.h - width * 2.0).max(0.0),
    };

    // inner corner radius shrunk by border width
    let inner_corner = CornerStyle {
        radius: (corner.radius - width).max(0.0),
        segments: corner.segments,
    };

    let inner_verts = perimeter_vertices(inner_rect, inner_corner, color_arr);

    let outer_count = outer_verts.len() as u32;
    let inner_count = inner_verts.len() as u32;

    // both rings must have same vertex count for simple stitching
    // they will if corner segments are the same — which they are
    assert_eq!(outer_count, inner_count, "outer/inner vertex count mismatch");

    let mut vertices = outer_verts;
    vertices.extend(inner_verts);

    // stitch outer ring (0..outer_count) to inner ring (outer_count..outer_count*2)
    let mut indices: Vec<u32> = Vec::new();
    let n = outer_count;

    for i in 0..n {
        let o0 = i;
        let o1 = (i + 1) % n;
        let i0 = n + i;
        let i1 = n + (i + 1) % n;

        // two triangles per quad strip segment
        indices.push(o0);
        indices.push(o1);
        indices.push(i0);

        indices.push(o1);
        indices.push(i1);
        indices.push(i0);
    }

    Mesh::new(MeshKind::Triangle, vertices, indices)
}

/// Shared perimeter vertex builder — used by both border types and quad
pub fn perimeter_vertices(rect: Rect, corner: CornerStyle, color: [f32; 4]) -> Vec<Vertex> {
    let radius = corner.radius.min(rect.w * 0.5).min(rect.h * 0.5);
    let segments = corner.segments.max(1);

    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.w;
    let y1 = rect.y + rect.h;

    let corners = [
        ([x0 + radius, y0 + radius], PI),             // top-left
        ([x1 - radius, y0 + radius], 3.0 * PI / 2.0), // top-right
        ([x1 - radius, y1 - radius], 0.0),             // bottom-right
        ([x0 + radius, y1 - radius], PI / 2.0),        // bottom-left
    ];

    let mut vertices = Vec::new();

    for ([cx, cy], start_angle) in &corners {
        for s in 0..=segments {
            let t = s as f32 / segments as f32;
            let angle = start_angle + t * (PI / 2.0);
            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius;
            vertices.push(Vertex::new([x, y, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], color));
        }
    }

    vertices
}