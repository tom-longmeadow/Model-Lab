use crate::ui::{
    
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
    macros::{impl_widget_base, impl_widget_text}, text::{item::TextItem, params::TextParam},
    widget::{Widget, WidgetBase, WidgetRole},
     widget_text::WidgetText
 
};

#[derive(Clone, Debug)]
pub struct Label {
    base: WidgetBase,
    text: WidgetText,
}

impl Label {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new(WidgetRole::Control),
            text: WidgetText::new(value),
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
        let padding = self.base.padding(params);
        let style = self.text.resolved_style(self.base.text_style(params));
        let s = measurer.measure(self.text.text(), &style);
        Size {
            w: (s.w + padding.left + padding.right).min(available.w),
            h: (s.h + padding.top + padding.bottom).min(available.h),
        }
    }

    fn collect_text_inner(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
        let rect = self.base.rect();
        let padding = self.base.padding(params);
        let style = self.text.resolved_style(self.base.text_style(params));
        out.push(TextParam::new(
            style,
            vec![TextItem {
                text: self.text().to_string(),
                x: rect.x + padding.left,
                y: rect.y + padding.top,
            }],
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