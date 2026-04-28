use crate::ui::{layout::color::Color, text::font::TextFont};

 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justified,
    End,
}


#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font: TextFont,
    pub color: Color,
    pub align: TextAlign,
}

impl TextStyle {
    pub fn new(
        font_size: f32,
        line_height: f32,
        font: TextFont,
        color: Color,
        align: TextAlign,
    ) -> Self {
        Self {
            font_size,
            line_height,
            font,
            color,
            align,
        }
    }

    
}


#[derive(Clone)]
pub struct TextStyleFactory {
    pub font: TextFont,
    pub color: Color,
    pub line_height_ratio: f32,
    pub align: TextAlign,
}

impl TextStyleFactory {
    pub fn new(font: TextFont, color: Color) -> Self {
        Self {
            font,
            color,
            line_height_ratio: 1.25,
            align: TextAlign::Left,
        }
    }

    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.line_height_ratio = ratio;
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }

    pub fn style(&self, font_size: f32) -> TextStyle {
        TextStyle::new(
            font_size,
            self.line_height(font_size),
            self.font.clone(),
            self.color,
            self.align,
        )
    }
}