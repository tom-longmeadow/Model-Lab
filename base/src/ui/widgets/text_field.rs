use crate::ui::{ 
    layout::{
            layout_params::LayoutParams,
            rect::Rect,
            size::Size,
            text_measurer::TextMeasurer,
    }, 
    macros::{impl_widget_base, impl_widget_text}, 
    text::{item::TextItem, params::TextParam}, 
    widget::{ControlKind, Widget, WidgetBase}, 
    widget_text::{TextKind, WidgetText} 
};

#[derive(Clone, Debug)]
pub struct TextField {
    base: WidgetBase,
    text: WidgetText,
    placeholder: String,
}

impl TextField {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new(ControlKind::TextField),
            text: WidgetText::new(value, TextKind::TextField),
            placeholder: String::new(),
        }
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    fn display_text(&self) -> &str {
        if self.text.text().is_empty() {
            &self.placeholder
        } else {
            self.text.text()
        }
    }
}

impl_widget_base!(TextField);
impl_widget_text!(TextField);

 
impl Widget for TextField {
    fn base(&self) -> &WidgetBase {
        &self.base
    }

    fn collect_rects_inner(&self, out: &mut Vec<WidgetBase>) {
        out.push(self.base);
    }

    fn collect_text_inner(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
        let rect    = self.base.rect();
        let padding = params.control.style_for(self.base.kind()).padding;
        let style   = params.text.style_for(self.text.kind());

        out.push(TextParam::new(
            style,
            vec![TextItem::new(self.display_text(), rect, padding)],
        ));
    }

    fn measure(
        &mut self,
        available: Size,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> Size {
        let padding = params.control.style_for(self.base.kind()).padding;
        let style   = params.text.style_for(self.text.kind());
        let s       = measurer.measure(self.display_text(), &style);
        Size {
            w: (s.w + padding.left + padding.right).min(available.w),
            h: (s.h + padding.top + padding.bottom).min(available.h),
        }
    }

    fn arrange(
        &mut self,
        rect: Rect,
        _params: &LayoutParams,
        _measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);
    }
}