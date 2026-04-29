 

use crate::ui::layout::{
    border::BorderStyle, 
    color::Color, 
    corner::CornerStyle, 
    edge_insets::EdgeInsets, 
    gap::Gap,
};
use crate::ui::text::font::TextFont;
use crate::ui::text::style::{TextAlign, TextStyle, TextStyleFactory};
use crate::ui::widget::ControlKind;
use crate::ui::widget_text::TextKind;


#[derive(Clone, Copy, Debug)]
pub struct ControlStyle {
    pub padding:    EdgeInsets,
    pub background: Color,
    pub border:     BorderStyle,
    pub corner:     CornerStyle,
}
#[derive(Clone, Copy, Debug)]
pub struct TextStyles {
    pub heading:     TextStyle,
    pub sub_heading: TextStyle,
    pub label:       TextStyle,
    pub caption:     TextStyle,
    pub textfield:   TextStyle,
}

impl TextStyles {
    pub fn new(base_size: f32) -> Self {
        let f = TextStyleFactory::new(TextFont::Regular, Color::WHITE)
            .with_align(TextAlign::Left)
            .with_ratio(1.25);

        let b = TextStyleFactory::new(TextFont::Bold, Color::WHITE)
            .with_align(TextAlign::Left)
            .with_ratio(1.25);

        let t = TextStyleFactory::new(TextFont::Regular, Color::WHITE)
            .with_align(TextAlign::Right)
            .with_ratio(1.25);

        Self {
            heading:     b.style(base_size * 1.2),
            sub_heading: b.style(base_size * 1.1),
            label:       f.style(base_size),
            caption:     f.style(base_size * 0.85),
            textfield:   t.style(base_size),
        }
    }

    pub fn style_for(&self, kind: TextKind) -> TextStyle {
        match kind {
            TextKind::Heading    => self.heading,
            TextKind::SubHeading => self.sub_heading,
            TextKind::Label      => self.label,
            TextKind::Caption    => self.caption,
            TextKind::TextField  => self.textfield,
        }
    }
}
 
 

#[derive(Clone, Copy, Debug)]
pub struct ControlStyles {
    pub label:     ControlStyle,
    pub textfield: ControlStyle,
    pub button:    ControlStyle,
    pub panel:     ControlStyle,
    pub flow:     ControlStyle,
}



impl ControlStyles {
    pub fn style_for(&self, kind: ControlKind) -> ControlStyle {
        match kind {
            ControlKind::Label     => self.label,
            ControlKind::TextField => self.textfield,
            ControlKind::Button    => self.button,
            ControlKind::Panel     => self.panel,
            ControlKind::Flow     => self.flow,
        }
    }
}

impl Default for ControlStyles {
    fn default() -> Self {
       

        let padding_small = EdgeInsets::vert_horz(8.0, 8.0);
        let padding_big = EdgeInsets::vert_horz(12.0, 12.0);
        let corner_small = CornerStyle::new(6.0, 6);
        let corner_big = CornerStyle::new(10.0, 10);


       
        Self {
            label: ControlStyle {
                padding:    padding_small,
                background: Color::TRANSPARENT,
                border:     BorderStyle::none(),
                corner:     CornerStyle::none(),
            },
            textfield: ControlStyle {
                padding:    padding_small,
                background: Color::GREY_60,
                border:     BorderStyle::line(Color::GREY_20),
                corner:     corner_small,
            },
            button: ControlStyle {
                padding:    padding_big,
                background: Color::GREY_50,
                border:     BorderStyle::none(),
                corner:     corner_small,
            },
            panel: ControlStyle {
                padding:    padding_big,
                background: Color::GREY_40,
                border:     BorderStyle::line(Color::BLACK),
                corner:     corner_big,
            },
            flow: ControlStyle {
                padding:    EdgeInsets::none(),
                background: Color::TRANSPARENT,
                border:     BorderStyle::none(),
                corner:     CornerStyle::none(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutParams {
    pub text:      TextStyles,
    pub control:   ControlStyles,
    pub flow:      Gap,
    pub base_size: f32,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self::with_base_size(32.0)
    }
}

impl LayoutParams {
    pub fn with_base_size(base_size: f32) -> Self {
        Self {
            text:      TextStyles::new(base_size),
            control:   ControlStyles::default(),
            flow:      Gap::all(8.0),
            base_size,
        }
    }

    pub fn with_font_scale(self, scale: f32) -> Self {
        Self::with_base_size(self.base_size * scale)
    }

    pub fn increase_font(self) -> Self { self.with_font_scale(1.1) }
    pub fn decrease_font(self) -> Self { self.with_font_scale(0.9) }
}