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
pub struct Label {
    base: WidgetBase,
    text: WidgetText, 
}

impl Label {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new(),
            text: WidgetText::new(value, TextStyle::default()), 
        }
    }  
}

impl_widget_base!(Label);
impl_widget_text!(Label);

impl Widget for Label {

    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size {
       self.text.measure_clamped(available, measurer)
    }

    fn arrange(&mut self, rect: Rect, _measurer: &mut dyn TextMeasurer) {
        let mut model = self.base.box_model();
        model.set_rect(rect);
        self.base.set_box_model(model);
    }
    
    fn collect_text(&self, out: &mut Vec<TextParam>) {

        let rect = self.base.box_model().rect();
        out.push(TextParam::new(
            self.text.style(),
            vec![TextItem {
                text: self.text().to_string(),
                x: rect.x,
                y: rect.y,
            }],
        )); 
    }

    fn collect_rects(&self, out: &mut Vec<BoxModel>) {
        out.push(self.base.box_model());
    }
}
 
 