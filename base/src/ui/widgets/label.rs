use crate::ui::{
    
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
    macros::{impl_widget_base, impl_widget_text}, text::{item::TextItem, params::TextParam},
    widget::{ControlKind, Widget, WidgetBase},
     widget_text::{TextKind, WidgetText}
 
};

#[derive(Clone, Debug)]
pub struct Label {
    base: WidgetBase,
    text: WidgetText,
}

impl Label {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new(ControlKind::Label),
            text: WidgetText::new(value, TextKind::Label),
        }
    }

     pub fn with_kind(text: impl Into<String>, kind: TextKind) -> Self {
        Self {
            base: WidgetBase::new(ControlKind::Label),
            text: WidgetText::new(text, kind),
        }
    }
}

impl_widget_base!(Label);
impl_widget_text!(Label);

impl Widget for Label {

    fn base(&self) -> &WidgetBase {
        &self.base
    }
    
    fn measure(
        &mut self,
        available: Size,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> Size {
        let padding = params.control.style_for(self.base.kind()).padding;
        let style   = params.text.style_for(self.text.kind());
        let s = measurer.measure(self.text.text(), &style);
        Size {
            w: (s.w + padding.left + padding.right).min(available.w),
            h: (s.h + padding.top + padding.bottom).min(available.h),
        }
    }

    fn collect_text_inner(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
        let rect = self.base.rect();
        let padding = params.control.style_for(self.base.kind()).padding;
        let style   = params.text.style_for(self.text.kind());
        out.push(TextParam::new(
            style,
            vec![TextItem::new(self.text(), rect, padding)],
        ));
    }

    fn arrange(
        &mut self,
        rect: Rect,
        _params: &LayoutParams,
        _measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);
    }

    
    fn collect_rects_inner(&self, out: &mut Vec<WidgetBase>) {
        out.push(self.base);
    }
}