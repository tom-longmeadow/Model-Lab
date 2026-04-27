use crate::ui::text::font::TextFont;

 

#[derive(Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font: TextFont,
    pub color: [u8; 4],
}

impl TextStyle {
    pub fn new(font_size: f32, line_height: f32, font: TextFont, color: [u8; 4]) -> Self {
        Self { font_size, line_height, font, color }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(24.0, 32.0, TextFont::Regular, [255, 255, 255, 255])
    }
}

#[derive(Clone)]
pub struct TextStyleFactory {
    pub font: TextFont,
    pub color: [u8; 4],
    pub line_height_ratio: f32,
}

impl TextStyleFactory {
    pub fn new(font: TextFont, color: [u8; 4]) -> Self {
        Self { font, color, line_height_ratio: 1.25 }
    }

    pub fn ui_defaults(color: [u8; 4]) -> Self {
        Self::new(TextFont::Regular, color).with_ratio(1.25)
    }

    pub fn spreadsheet_defaults(color: [u8; 4]) -> Self {
        Self::new(TextFont::Regular, color).with_ratio(1.20)
    }

    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.line_height_ratio = ratio;
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
        )
    }
}
