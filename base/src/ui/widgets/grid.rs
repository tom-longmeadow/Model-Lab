use crate::ui::{ 
    container::WidgetContainer, 
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
    macros::{impl_widget_base, impl_widget_container}, 
    text::params::TextParam, widget::{Widget, WidgetBase, WidgetRole, collect_rects, collect_text}

    
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
            base: WidgetBase::new(WidgetRole::Container),
            container: WidgetContainer::new(),
            columns: columns.max(1),
        }
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1); 
    }
}

impl_widget_base!(Grid);
impl_widget_container!(Grid);

impl Widget for Grid {

    fn base(&self) -> &WidgetBase {
        &self.base
    }
    
    fn measure(
        &mut self,
        available: Size,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> Size {
        let cols = self.columns.max(1);
        let count = self.container.children().len();
        if count == 0 { return Size { w: 0.0, h: 0.0 }; }

        let gap = self.container.resolved_gap(params.gap);
        let rows = count.div_ceil(cols);

        // 1) measure each child at unconstrained width to get natural size
        let mut col_widths = vec![0.0f32; cols];
        let mut row_heights = vec![0.0f32; rows];

        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let s = child.measure(Size { w: f32::MAX, h: available.h }, params, measurer);
            let col = i % cols;
            let row = i / cols;
            col_widths[col] = col_widths[col].max(s.w);
            row_heights[row] = row_heights[row].max(s.h);
            i += 1;
        });

        let total_w = col_widths.iter().sum::<f32>() + gap * (cols.saturating_sub(1) as f32);
        let total_h = row_heights.iter().sum::<f32>() + gap * (rows.saturating_sub(1) as f32);

        Size { w: total_w.min(available.w), h: total_h.min(available.h) }
    }

    fn arrange(
        &mut self,
        rect: Rect,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);

        let cols = self.columns.max(1);
        let count = self.container.children().len();
        if count == 0 { return; }

        let gap = self.container.resolved_gap(params.gap);
        let rows = count.div_ceil(cols);

        // 1) measure natural column widths
        let mut col_widths = vec![0.0f32; cols];
        let mut row_heights = vec![0.0f32; rows];

        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let s = child.measure(Size { w: f32::MAX, h: rect.h }, params, measurer);
            let col = i % cols;
            let row = i / cols;
            col_widths[col] = col_widths[col].max(s.w);
            row_heights[row] = row_heights[row].max(s.h);
            i += 1;
        });

        // 2) arrange using natural widths
        let mut i = 0usize;
        self.container.for_each_child_mut(|child| {
            let col = i % cols;
            let row = i / cols;

            let x = rect.x + col_widths.iter().take(col).sum::<f32>() + col as f32 * gap;
            let y = rect.y + row_heights.iter().take(row).sum::<f32>() + row as f32 * gap;

            child.arrange(
                Rect { x, y, w: col_widths[col], h: row_heights[row] },
                params,
                measurer,
            );
            i += 1;
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