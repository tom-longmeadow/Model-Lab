use crate::ui::{ 
    container::WidgetContainer, 
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
    macros::{impl_widget_base, impl_widget_container}, 
    text::params::TextParam, widget::{ControlKind, Widget, WidgetBase, collect_rects, collect_text}
    
};

#[derive(Debug)]
pub struct Panel {
    base: WidgetBase,
    container: WidgetContainer,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(ControlKind::Panel),
            container: WidgetContainer::new(),
        }
    }
}

impl_widget_base!(Panel);
impl_widget_container!(Panel);

impl Widget for Panel {

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
        let inner = Size {
            w: (available.w - padding.left - padding.right).max(0.0),
            h: (available.h - padding.top - padding.bottom).max(0.0),
        };

        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;

        self.container.for_each_child_mut(|child| {
            let s = child.measure(inner, params, measurer);
            max_w = max_w.max(s.w);
            max_h = max_h.max(s.h);
        });

        Size {
            w: (max_w + padding.left + padding.right).min(available.w),
            h: (max_h + padding.top + padding.bottom).min(available.h),
        }
    }

    fn arrange(
        &mut self,
        rect: Rect,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);

        let padding = params.control.style_for(self.base.kind()).padding;
        let inner = Rect {
            x: rect.x + padding.left,
            y: rect.y + padding.top,
            w: (rect.w - padding.left - padding.right).max(0.0),
            h: (rect.h - padding.top - padding.bottom).max(0.0),
        };

        self.container.for_each_child_mut(|child| {
            child.arrange(inner, params, measurer);
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
