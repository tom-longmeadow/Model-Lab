use crate::ui::{ 
    container::WidgetContainer, 
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
    macros::{impl_widget_base, impl_widget_container}, 
    text::params::TextParam, widget::{Widget, WidgetBase, WidgetRole, collect_rects, collect_text} 
    
};

#[derive(Debug)]
pub struct Row {
    base: WidgetBase,
    container: WidgetContainer,
}

impl Row {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(WidgetRole::Container),
            container: WidgetContainer::new(),
        }
    }
}

impl_widget_base!(Row);
impl_widget_container!(Row);

impl Widget for Row {

    fn base(&self) -> &WidgetBase {
        &self.base
    }
    
    fn measure(
        &mut self,
        available: Size,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> Size {
        let count = self.container.children().len();
        let gap = self.container.resolved_gap(params.gap);

        let mut i = 0usize;
        let mut w = 0.0f32;
        let mut h = 0.0f32;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(available, params, measurer);
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

    fn arrange(
        &mut self,
        rect: Rect,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);

       let gap = self.container.resolved_gap(params.gap);
        let mut x = rect.x;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(Size { w: rect.w, h: rect.h }, params, measurer);

            child.arrange(
                Rect {
                    x,
                    y: rect.y,
                    w: s.w.min(rect.w),
                    h: rect.h,
                },
                params,
                measurer,
            );

            x += s.w + gap;
        });
    }

    fn collect_rects_inner(&self, out: &mut Vec<WidgetBase>) {
        out.push(self.base);
        for child in self.container.children() {
            collect_rects(child.as_ref(), out);
        }
    }

    fn collect_text_inner(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
        for child in self.container.children() {
            collect_text(child.as_ref(), out, params);
        }
    }

    
}