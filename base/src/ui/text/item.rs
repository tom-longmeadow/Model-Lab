use crate::ui::{
    layout::{edge_insets::EdgeInsets, rect::Rect}, text::style::TextAlign
};

 

 

pub struct TextItem {
    pub text: String,
    pub x: f32,
    pub y: f32, 
    pub width: f32,
}


impl TextItem {
    pub fn new(text: impl Into<String>, rect: Rect, padding: EdgeInsets) -> Self {
        Self {
            text: text.into(),
            x: rect.x + padding.left,
            y: rect.y + padding.top,
            width: rect.w - padding.left - padding.right,
        }
    }
}