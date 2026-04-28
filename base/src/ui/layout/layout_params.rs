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

impl Default for TextStyles {
    fn default() -> Self { 

        let f = TextStyleFactory::new(
            TextFont::Regular, Color::WHITE
        )
        .with_align(TextAlign::Left)
        .with_ratio(1.25);

        let t = TextStyleFactory::new(
            TextFont::Regular, Color::WHITE
        )
        .with_align(TextAlign::Right)
        .with_ratio(1.25);
        
        Self {
            heading:     f.style(48.0),
            sub_heading: f.style(32.0),
            label:       f.style(24.0),
            caption:     f.style(24.0),
            textfield:   t.style(24.0),
        }
    }
}

impl TextStyles {
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
            ControlKind::Flow     => self.panel,
        }
    }
}

impl Default for ControlStyles {
    fn default() -> Self {
        Self {
            label: ControlStyle {
                padding:    EdgeInsets::new(4.0, 8.0, 4.0, 8.0),
                background: Color::TRANSPARENT,
                border:     BorderStyle::none(),
                corner:     CornerStyle::none(),
            },
            textfield: ControlStyle {
                padding:    EdgeInsets::new(4.0, 8.0, 4.0, 8.0),
                background: Color::GREY_60,
                border:     BorderStyle::none(),
                corner:     CornerStyle::new(4.0, 8),
            },
            button: ControlStyle {
                padding:    EdgeInsets::new(6.0, 12.0, 6.0, 12.0),
                background: Color::GREY_50,
                border:     BorderStyle::none(),
                corner:     CornerStyle::new(4.0, 8),
            },
            panel: ControlStyle {
                padding:    EdgeInsets::all(10.0),
                background: Color::GREY_40,
                border:     BorderStyle::none(),
                corner:     CornerStyle::new(6.0, 8),
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
    pub text:    TextStyles,
    pub control: ControlStyles,
    pub flow:    Gap,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            text:    TextStyles::default(),
            control: ControlStyles::default(),
            flow:    Gap::all(6.0),
        }
    }
}

impl LayoutParams {
    pub fn new(text: TextStyles, control: ControlStyles, flow: Gap) -> Self {
        Self { text, control, flow }
    }
}