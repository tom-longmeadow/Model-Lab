use crate::ui::{
    text::params::TextParam,
    widget::{
        Widget, WidgetBase, WidgetId, r#box::BoxModel, 
        container::{WidgetContainer}, 
        layout::{rect::Rect, size::Size, text_measurer::TextMeasurer}, 
        macros::{impl_widget_base, impl_widget_container}, 
        
    },
};

#[derive(Debug)] 
pub struct Row {
    base: WidgetBase,
    container: WidgetContainer,
}

impl Row {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(),
            container: WidgetContainer::new(),
        }
    }
}

impl_widget_base!(Row); 
impl_widget_container!(Row);

impl Widget for Row {
    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size {
        let count = self.container.children().len();
        let gap = self.container.gap();

        let mut i = 0usize;
        let mut w = 0.0f32;
        let mut h = 0.0f32;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(available, measurer);
            w += s.w;
            h = h.max(s.h);

            if i + 1 < count {
                w += gap;
            }
            i += 1;
        });

        Size {
            w: w.min(available.w),
            h: h.min(available.h),
        }
    }

    fn arrange(&mut self, rect: Rect, measurer: &mut dyn TextMeasurer) {
        let mut model = self.base.box_model();
        model.set_rect(rect);
        self.base.set_box_model(model);

        let gap = self.container.gap();
        let mut x = rect.x;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(Size { w: rect.w, h: rect.h }, measurer);

            child.arrange(
                Rect {
                    x,
                    y: rect.y,
                    w: s.w.min(rect.w),
                    h: rect.h,
                },
                measurer,
            );

            x += s.w + gap;
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