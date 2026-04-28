 

use crate::{
    mesh::{Mesh, MeshKind, ui::border::perimeter_vertices, vertex::Vertex},
    ui::layout::{color::Color, corner::CornerStyle, rect::Rect},
};

 

pub fn quad(rect: Rect, color: Color, corner: CornerStyle) -> Mesh {
    if corner.radius <= 0.0 {
        quad_square(rect, color)
    } else {
        quad_rounded(rect, color, corner)
    }
}


fn quad_square(rect: Rect, color: Color) -> Mesh {
    let [r, g, b, a] = color.as_f32_array();

    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.w;
    let y1 = rect.y + rect.h;

    let vertices = vec![
        Vertex::new([x0, y0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], [r, g, b, a]),
        Vertex::new([x1, y0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0], [r, g, b, a]),
        Vertex::new([x1, y1, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0], [r, g, b, a]),
        Vertex::new([x0, y1, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0], [r, g, b, a]),
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    Mesh::new(MeshKind::Triangle, vertices, indices)
}


fn quad_rounded(rect: Rect, color: Color, corner: CornerStyle) -> Mesh {
    let [r, g, b, a] = color.as_f32_array();
    let color_arr = [r, g, b, a];

    let vertices = perimeter_vertices(rect, corner, color_arr);
    let n = vertices.len() as u32;

    let mut indices: Vec<u32> = Vec::new();
    for i in 1..(n - 1) {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    Mesh::new(MeshKind::Triangle, vertices, indices)
}