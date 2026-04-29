 
use base::ui::{layout::{size::Size, text_measurer::TextMeasurer}, text::{font::TextFont, style::TextStyle}};
use glyphon::cosmic_text;

use crate::renderer::pass::text::font::TextFontExt;


pub struct GlyphonTextMeasurer {
    font_system: glyphon::FontSystem,
}

impl GlyphonTextMeasurer {
    pub fn new() -> Self {
        let mut font_system = glyphon::FontSystem::new();
        for font in TextFont::all() {
            font_system.db_mut().load_font_data(font.font_bytes().to_vec());
        }
        Self { font_system }
    }
}

impl TextMeasurer for GlyphonTextMeasurer {
    fn measure(&mut self, text: &str, style: &TextStyle) -> Size {
        let attrs = style.font.attrs();

        let mut buffer = glyphon::Buffer::new(
            &mut self.font_system,
            glyphon::Metrics::new(style.font_size, style.line_height),
        );

        // always measure left-aligned at unconstrained width
        // alignment affects rendering position, not natural content size
        buffer.set_size(&mut self.font_system, Some(f32::MAX), None);
        buffer.set_text(
            &mut self.font_system,
            text,
            &attrs,
            glyphon::Shaping::Advanced,
            Some(cosmic_text::Align::Left),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let w = buffer
            .layout_runs()
            .map(|r| r.line_w)
            .fold(0.0f32, f32::max)
            .ceil();  // ceil prevents sub-pixel wrapping at word boundaries

        let h = buffer
            .layout_runs()
            .count() as f32 * style.line_height;

        Size { w, h } 
    }
}
 