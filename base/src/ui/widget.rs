
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ui::{ 
    layout::{
        border::BorderStyle, color::Color, corner::CornerStyle, edge_insets::EdgeInsets, layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer
    }, text::{params::TextParam, style::TextStyle}, 

};

 
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

impl WidgetId {
    pub fn next() -> Self {
        Self(NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::next()
    }
}

pub trait Widget {
    fn measure(&mut self, available: Size, params: &LayoutParams, measurer: &mut dyn TextMeasurer) -> Size;
    fn arrange(&mut self, rect: Rect, params: &LayoutParams, measurer: &mut dyn TextMeasurer);
    fn base(&self) -> &WidgetBase;

    fn collect_rects_inner(&self, _out: &mut Vec<WidgetBase>) {}
    fn collect_text_inner(&self, _out: &mut Vec<TextParam>, _params: &LayoutParams) {}
}

pub fn collect_rects(widget: &dyn Widget, out: &mut Vec<WidgetBase>) {
    if widget.base().is_visible() {
        widget.collect_rects_inner(out);
    }
}

pub fn collect_text(widget: &dyn Widget, out: &mut Vec<TextParam>, params: &LayoutParams) {
    if widget.base().is_visible() {
        widget.collect_text_inner(out, params);
    }
}
 
// pub trait Widget {
//     fn measure(&mut self, available: Size, params: &LayoutParams, measurer: &mut dyn TextMeasurer) -> Size;
//     fn arrange(&mut self, rect: Rect, params: &LayoutParams, measurer: &mut dyn TextMeasurer);
//     fn base(&self) -> &WidgetBase;

//     fn collect_text(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
//         if self.base().is_visible() {
//             self.collect_text_inner(out, params);
//         }
//     }

//     fn collect_rects(&self, out: &mut Vec<WidgetBase>) {
//         if self.base().is_visible() {
//             self.collect_rects_inner(out);
//         }
//     }

//     fn collect_text_inner(&self, _out: &mut Vec<TextParam>, _params: &LayoutParams) {}
//     fn collect_rects_inner(&self, _out: &mut Vec<WidgetBase>) {}
// }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WidgetRole {
    #[default]
    Panel,
    Container,
    Control,
    TextField,
}

#[derive(Clone, Copy, Debug)]
pub struct WidgetBase {
    id: WidgetId,
    visible: bool,
    role: WidgetRole,
    rect: Rect, 
    background: Option<Color>,
    border: Option<BorderStyle>,
}

impl WidgetBase {
    pub fn new(role: WidgetRole) -> Self {
        Self {
            id: WidgetId::next(),
            visible: true,
            role,
            rect: Rect::default(), 
            background: None,
            border: None,
        }
    }

    pub fn id(&self) -> WidgetId { self.id }
    pub fn role(&self) -> WidgetRole { self.role }
    pub fn is_visible(&self) -> bool { self.visible }
    pub fn rect(&self) -> Rect { self.rect }
    pub fn set_rect(&mut self, rect: Rect) { self.rect = rect; }

    pub fn set_visible(&mut self, visible: bool) { self.visible = visible; }
    pub fn hide(&mut self) { self.visible = false; }
    pub fn show(&mut self) { self.visible = true; }

    
    pub fn background(&self) -> Option<Color> { self.background }
    pub fn set_background(&mut self, color: Color) { self.background = Some(color); }
    pub fn resolved_background(&self, fallback: Color) -> Color {
        self.background.unwrap_or(fallback)
    }
    pub fn clear_background(&mut self) {
        self.background = None;
    }

    pub fn border(&self) -> Option<BorderStyle> { self.border }
    pub fn set_border(&mut self, border: BorderStyle) { self.border = Some(border); }
    pub fn resolved_border(&self, fallback: BorderStyle) -> BorderStyle {
        self.border.unwrap_or(fallback)
    }
    pub fn clear_border(&mut self) {
        self.border = None;
    }

    pub fn text_style(&self, params: &LayoutParams) -> TextStyle {
        params.text_for(self.role)
    }

    pub fn background_color(&self, params: &LayoutParams) -> Color {
        params.background_for(self.role)
    }

    pub fn border_style(&self, params: &LayoutParams) -> BorderStyle {
        params.border_for(self.role)
    }

    pub fn corner_style(&self, params: &LayoutParams) -> CornerStyle {
        params.corner_for(self.role)
    }

    pub fn padding(&self, params: &LayoutParams) -> EdgeInsets {
        params.padding_for(self.role)
    }

   

}