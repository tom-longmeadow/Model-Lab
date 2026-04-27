use crate::ui::{
    text::{
        item::TextItem,
        params::TextParam,
        style::TextStyle,
    },
    widget::{
        Widget, WidgetBase, WidgetId, r#box::BoxModel, 
        layout::{rect::Rect, size::Size, text_measurer::TextMeasurer}, 
        macros::{impl_widget_base, impl_widget_text}, text::WidgetText
    },
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
            base: WidgetBase::new(),
            text: WidgetText::new(value, TextStyle::default()),
            placeholder: String::new(),
        }
    } 

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
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

    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size {
        if self.text.text().is_empty() && !self.placeholder.is_empty() {
            let s = measurer.measure(&self.placeholder, &self.text.style());
            Size {
                w: s.w.min(available.w),
                h: s.h.min(available.h),
            }
        } else {
            self.text.measure_clamped(available, measurer)
        }
    }

    fn arrange(&mut self, rect: Rect, _measurer: &mut dyn TextMeasurer) {
        let mut model = self.base.box_model(); // copy
        model.set_rect(rect);
        self.base.set_box_model(model); // write back through setter
    }

    
    fn collect_text(&self, out: &mut Vec<TextParam>) {
        let rect = self.base.box_model().rect();
        out.push(TextParam::new(
            self.text.style(),
            vec![TextItem {
                text: self.display_text().to_string(),
                x: rect.x,
                y: rect.y,
            }],
        ));
    }

    fn collect_rects(&self, out: &mut Vec<BoxModel>) {
        out.push(self.base.box_model());
    }
}