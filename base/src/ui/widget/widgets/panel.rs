use crate::ui::{
    text::params::TextParam,
    widget::{
        Widget, WidgetBase, WidgetId, r#box::BoxModel, container::WidgetContainer, 
        layout::{rect::Rect, size::Size, text_measurer::TextMeasurer}, 
        macros::{impl_widget_base, impl_widget_container}
    },
};

#[derive(Debug)]
pub struct Panel {
    base: WidgetBase,
    container: WidgetContainer,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(),
            container: WidgetContainer::new(),
        }
    } 
}

impl_widget_base!(Panel);
impl_widget_container!(Panel);

impl Widget for Panel {
    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size {
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(available, measurer);
            max_w = max_w.max(s.w);
            max_h = max_h.max(s.h);
        });

        Size {
            w: max_w.min(available.w),
            h: max_h.min(available.h),
        }
    }

    fn arrange(&mut self, rect: Rect, measurer: &mut dyn TextMeasurer) {
        let mut model = self.base.box_model();
        model.set_rect(rect);
        self.base.set_box_model(model);

        self.container.for_each_child_mut(|child| {
            child.arrange(rect, measurer);
        });
    }

    fn collect_text(&self, out: &mut Vec<TextParam>) {
        for child in self.container.children() {
            child.collect_text(out);
        }
    }

    fn collect_rects(&self, out: &mut Vec<BoxModel>) {
        out.push(self.base.box_model());
        for child in self.container.children() {
            child.collect_rects(out);
        }
    }
}