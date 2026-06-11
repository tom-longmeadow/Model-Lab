 

use base::ui::text::{font::{FontStyle, TextFont}, style::TextAlign};
 

use glyphon::{
    cosmic_text, 
};


pub trait TextFontExt {
    fn attrs(&self) -> glyphon::Attrs<'_>;
}

impl TextFontExt for TextFont {
    fn attrs(&self) -> glyphon::Attrs<'_> {
        let weight = cosmic_text::Weight(self.weight().0);
        let style = match self.font_style() {
            FontStyle::Italic => cosmic_text::Style::Italic,
            FontStyle::Normal => cosmic_text::Style::Normal,
        };

        glyphon::Attrs::new()
            .family(cosmic_text::Family::Name(self.family_name()))
            .weight(weight)
            .style(style)
    }
}


pub fn to_glyphon_align(align: TextAlign) -> cosmic_text::Align {
    match align {
        TextAlign::Left => cosmic_text::Align::Left,
        TextAlign::Center => cosmic_text::Align::Center,
        TextAlign::Right => cosmic_text::Align::Right,
        TextAlign::Justified => cosmic_text::Align::Justified,
        TextAlign::End => cosmic_text::Align::End,
    }
}
