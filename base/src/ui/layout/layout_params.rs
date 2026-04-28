use crate::ui::{
    layout::{border::BorderStyle, color::Color, corner::CornerStyle, edge_insets::EdgeInsets}, 
    text::{font::TextFont, style::{TextAlign, TextStyle}}, widget::WidgetRole
};


#[derive(Clone, Copy, Debug)]
pub struct Padding {
    pub panel: EdgeInsets,
    pub control: EdgeInsets,
    pub textfield: EdgeInsets,
}


impl Default for Padding {
    fn default() -> Self {
        Self { 
            panel:     EdgeInsets::all(10.0),
            control:   EdgeInsets::new(4.0, 8.0, 4.0, 8.0),
            textfield: EdgeInsets::new(4.0, 8.0, 4.0, 8.0),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Corner {
    pub panel: CornerStyle,
    pub control: CornerStyle,  // used by both Control and TextField
}


impl Default for Corner {
    fn default() -> Self {
        Self { 
            panel: CornerStyle::new(6.0, 8), 
            control: CornerStyle::new(4.0, 8), 
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextStyles {
    pub label: TextStyle,
    pub textfield: TextStyle,
}

impl Default for TextStyles {
    fn default() -> Self {
        Self { 
            label: TextStyle::new(24.0, 32.0, 
                TextFont::Regular, Color::GREY_220, 
                TextAlign::Left), 
            textfield: TextStyle::new(24.0, 32.0, 
                TextFont::Regular, Color::WHITE, 
                TextAlign::Right),
        }
    }
}




#[derive(Clone, Copy, Debug)]
pub struct Borders {
    pub panel: BorderStyle,
    pub control: BorderStyle,
    pub textfield: BorderStyle,
}

impl Default for Borders {
    fn default() -> Self {
        Self { 
            panel: BorderStyle::none(), 
            control: BorderStyle::none(), 
            textfield: BorderStyle::line(Color::BLUE) 
        }
    }
}



#[derive(Clone, Copy, Debug)]
pub struct Background {
    pub panel: Color,
    pub container: Color,
    pub control: Color,
    pub textfield: Color,
}

impl Default for Background {
    fn default() -> Self {
        Self { 
            panel: Color::GREY_40,
            container: Color::TRANSPARENT,
             control: Color::GREY_50,
            textfield: Color::GREY_80,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutParams {
    pub padding: Padding,
    pub background: Background,
    pub gap: f32,          // containers only — single value, no struct needed
    pub corner: Corner,
    pub text: TextStyles,
    pub border: Borders,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            padding: Padding::default(),
            background: Background::default(),
            gap: Padding::default().panel.left,
            corner: Corner::default(),
            text: TextStyles::default(),
            border: Borders::default(), 
        }
    }
}

impl LayoutParams {
    pub fn padding_for(&self, role: WidgetRole) -> EdgeInsets {
        match role {
            WidgetRole::Panel     => self.padding.panel,
            WidgetRole::Control   => self.padding.control,
            WidgetRole::TextField => self.padding.textfield,
            WidgetRole::Container => unreachable!("containers do not use padding"),
        }
    }

    pub fn corner_for(&self, role: WidgetRole) -> CornerStyle {
        match role {
            WidgetRole::Panel     => self.corner.panel,
            WidgetRole::Control   => self.corner.control,
            WidgetRole::TextField => self.corner.control,
            WidgetRole::Container => self.corner.control,
        }
    }

     pub fn background_for(&self, role: WidgetRole) -> Color {
        match role {
            WidgetRole::Panel     => self.background.panel,
            WidgetRole::Control   => self.background.control,
            WidgetRole::TextField => self.background.textfield,
            WidgetRole::Container => self.background.container,
        }
    }

    pub fn border_for(&self, role: WidgetRole) -> BorderStyle {
        match role {
            WidgetRole::Panel     => self.border.panel,
            WidgetRole::Control   => self.border.control,
            WidgetRole::TextField => self.border.textfield,
            WidgetRole::Container => self.border.control,
        }
    }

    pub fn text_for(&self, role: WidgetRole) -> TextStyle {
        match role {
            WidgetRole::Control   => self.text.label,
            WidgetRole::TextField => self.text.textfield,
            WidgetRole::Panel     => unreachable!("panel does not use text style"),
            WidgetRole::Container => unreachable!("container does not use text style"),
        }
    }
}