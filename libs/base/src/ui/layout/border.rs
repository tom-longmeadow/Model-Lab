use crate::ui::layout::color::Color;


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BorderKind {
    #[default]
    None,
    Solid,  // triangle mesh, has width
    Line,   // line mesh, no width
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderStyle {
    pub color: Color,
    pub width: f32, 
    pub kind: BorderKind,
}

 
impl BorderStyle { 

    pub fn none() -> Self {
        Self { color: Color::BLACK, width: 0.0, kind: BorderKind::None }
    }
    
    pub fn solid(color: Color, width: f32) -> Self {
        Self { color, width, kind: BorderKind::Solid }
    }

    pub fn line(color: Color) -> Self {
        Self { color, width: 0.0, kind: BorderKind::Line }
    }

    pub fn has_border(&self) -> bool {
        self.color.is_visible() && match self.kind {
            BorderKind::Solid => self.width > 0.0,
            BorderKind::Line  => true, 
            BorderKind::None  => false, 
        }
    }

    pub fn is_solid(&self) -> bool {
        self.kind == BorderKind::Solid
    }

    pub fn is_line(&self) -> bool {
        self.kind == BorderKind::Line
    }

    pub fn is_none(&self) -> bool {
        self.kind == BorderKind::None
    }
}