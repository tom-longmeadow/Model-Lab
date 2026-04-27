use crate::ui::{
    text::params::TextParam,
    widget::{
        Widget, WidgetBase, WidgetId, r#box::BoxModel, 
        container::{WidgetContainer}, 
        layout::{rect::Rect, size::Size, text_measurer::TextMeasurer}, 
        macros::{impl_widget_base, impl_widget_container}
    },
};
#[derive(Debug)]
pub struct Grid {
    base: WidgetBase,
    container: WidgetContainer,
    columns: usize,
}

impl Grid {
    pub fn new(columns: usize) -> Self {
        Self {
            base: WidgetBase::new(),
            container: WidgetContainer::new(),
            columns: columns.max(1),
        }
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1);
        self.container.set_dirty(true);
    }
}

impl_widget_base!(Grid);
impl_widget_container!(Grid);

impl Widget for Grid {
    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size {
        let cols = self.columns.max(1);
        let count = self.container.children().len();
        if count == 0 {
            return Size { w: 0.0, h: 0.0 };
        }

        let gap = self.container.gap();
        let rows = count.div_ceil(cols);

        let total_gap_w = gap * (cols.saturating_sub(1) as f32);
        let cell_w = ((available.w - total_gap_w).max(0.0)) / cols as f32;

        let mut row_heights = vec![0.0f32; rows];

        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let s = child.measure(
                Size {
                    w: cell_w,
                    h: available.h,
                },
                measurer,
            );
            let row = i / cols;
            row_heights[row] = row_heights[row].max(s.h);
            i += 1;
        });

        let content_h: f32 = row_heights.iter().sum();
        let total_gap_h = gap * (rows.saturating_sub(1) as f32);

        Size {
            w: available.w,
            h: (content_h + total_gap_h).min(available.h),
        }
    }

    fn arrange(&mut self, rect: Rect, measurer: &mut dyn TextMeasurer) {
        let mut model = self.base.box_model();
        model.set_rect(rect);
        self.base.set_box_model(model);

        let cols = self.columns.max(1);
        let count = self.container.children().len();
        if count == 0 {
            return;
        }

        let gap = self.container.gap();
        let rows = count.div_ceil(cols);

        let total_gap_w = gap * (cols.saturating_sub(1) as f32);
        let cell_w = ((rect.w - total_gap_w).max(0.0)) / cols as f32;

        // First pass: measure and compute row heights.
        let mut measured = vec![Size { w: 0.0, h: 0.0 }; count];
        let mut row_heights = vec![0.0f32; rows];

        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let s = child.measure(
                Size {
                    w: cell_w,
                    h: rect.h,
                },
                measurer,
            );
            measured[i] = s;
            let row = i / cols;
            row_heights[row] = row_heights[row].max(s.h);
            i += 1;
        });

        // Second pass: arrange into grid cells.
        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let row = i / cols;
            let col = i % cols;

            let x = rect.x + col as f32 * (cell_w + gap);
            let y = rect.y
                + row_heights
                    .iter()
                    .take(row)
                    .sum::<f32>()
                + row as f32 * gap;

            let h = measured[i].h.min(row_heights[row]);

            child.arrange(
                Rect {
                    x,
                    y,
                    w: cell_w,
                    h,
                },
                measurer,
            );

            i += 1;
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