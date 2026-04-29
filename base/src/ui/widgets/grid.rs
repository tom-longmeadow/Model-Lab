use crate::ui::{ 
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer},
    macros::impl_widget_base,
    text::params::TextParam,
    widget::{ControlKind, Widget, WidgetBase, collect_rects, collect_text},
};

 
pub struct GridCell {
    pub widget: Box<dyn Widget>,
    pub span:   usize,
}

 
pub struct GridContainer {
    cells: Vec<GridCell>,
}

impl GridContainer {
    pub fn new() -> Self { Self { cells: Vec::new() } }

    pub fn push(&mut self, widget: Box<dyn Widget>, span: usize) {
        self.cells.push(GridCell { widget, span: span.max(1) });
    }

    pub fn cells(&self) -> &[GridCell] { &self.cells }

    pub fn cells_mut(&mut self) -> impl Iterator<Item = &mut GridCell> {
        self.cells.iter_mut()
    }

    pub fn clear(&mut self) { self.cells.clear(); }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(&mut GridCell)) {
        for cell in &mut self.cells { f(cell); }
    }
}

impl Default for GridContainer {
    fn default() -> Self { Self::new() }
}

 
pub struct Grid {
    base:      WidgetBase,
    container: GridContainer,
    columns:   usize,
}

impl Grid {
    pub fn new(columns: usize) -> Self {
        Self {
            base:      WidgetBase::new(ControlKind::Flow),
            container: GridContainer::new(),
            columns:   columns.max(1),
        }
    }

    pub fn push(&mut self, widget: Box<dyn Widget>) {
        self.container.push(widget, 1);
    }

    pub fn push_spanning(&mut self, widget: Box<dyn Widget>, span: usize) {
        self.container.push(widget, span.min(self.columns).max(1));
    }

    pub fn columns(&self) -> usize { self.columns }

    pub fn set_columns(&mut self, columns: usize) { self.columns = columns.max(1); }

    fn compute_col_widths(
        &mut self,
        available_h: f32,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> (Vec<f32>, Vec<f32>) {
        let cols  = self.columns;
        let h_gap = params.flow.horizontal; 

        let mut col_widths: Vec<f32> = vec![0.0; cols];

        // pass 1 — span=1 cells establish natural column widths
        let mut col = 0usize;
        for cell in self.container.cells_mut() {
            if cell.span == 1 {
                let s = cell.widget.measure(Size { w: f32::MAX, h: available_h }, params, measurer);
                col_widths[col] = col_widths[col].max(s.w);
            }
            col += cell.span;
            if col >= cols { col = 0; }
        }

        // pass 2 — partial-span cells expand columns if needed (not full-row spans)
        col = 0;
        for cell in self.container.cells_mut() {
            if cell.span > 1 && cell.span < cols {
                let s = cell.widget.measure(Size { w: f32::MAX, h: available_h }, params, measurer);
                let spanned_w = (0..cell.span)
                    .map(|i| col_widths.get(col + i).copied().unwrap_or(0.0))
                    .sum::<f32>()
                    + h_gap * (cell.span.saturating_sub(1) as f32);
                if s.w > spanned_w {
                    let excess = s.w - spanned_w;
                    let target = (0..cell.span)
                        .find(|&i| col_widths.get(col + i).copied().unwrap_or(0.0) == 0.0)
                        .unwrap_or(cell.span - 1);
                    col_widths[col + target] += excess;
                }
            }
            col += cell.span;
            if col >= cols { col = 0; }
        }

        // row heights — one pass, measure at actual cell width
        let mut row_heights: Vec<f32> = Vec::new();
        col = 0;
        let mut row = 0usize;
        for cell in self.container.cells_mut() {
            if row >= row_heights.len() { row_heights.push(0.0); }

            let cell_w = (0..cell.span)
                .map(|i| col_widths.get(col + i).copied().unwrap_or(0.0))
                .sum::<f32>()
                + h_gap * (cell.span.saturating_sub(1) as f32);

            let s = cell.widget.measure(Size { w: cell_w, h: available_h }, params, measurer);
            row_heights[row] = row_heights[row].max(s.h);

            col += cell.span;
            if col >= cols { col = 0; row += 1; }
        }

        (col_widths, row_heights)
    }
}

impl_widget_base!(Grid);

impl Widget for Grid {
    fn base(&self) -> &WidgetBase { &self.base }

    

    fn measure(
        &mut self,
        available: Size,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) -> Size {
        let cols  = self.columns;
        let h_gap = params.flow.horizontal;
        let v_gap = params.flow.vertical;

        let (col_widths, row_heights) = self.compute_col_widths(available.h, params, measurer);

        let total_w = col_widths.iter().sum::<f32>() + h_gap * (cols.saturating_sub(1) as f32);
        let total_h = row_heights.iter().sum::<f32>() + v_gap * (row_heights.len().saturating_sub(1) as f32);

        Size { w: total_w.min(available.w), h: total_h.min(available.h) }
    }

    fn arrange(
        &mut self,
        rect: Rect,
        params: &LayoutParams,
        measurer: &mut dyn TextMeasurer,
    ) {
        self.base.set_rect(rect);

        let cols  = self.columns;
        let h_gap = params.flow.horizontal;
        let v_gap = params.flow.vertical;

        let (col_widths, row_heights) = self.compute_col_widths(rect.h, params, measurer);

        // arrange pass
        let mut col = 0usize;
        let mut row = 0usize;

        for cell in self.container.cells_mut() {
            if row >= row_heights.len() { break; }

            let x = rect.x + col_widths.iter().take(col).sum::<f32>() + col as f32 * h_gap;
            let y = rect.y + row_heights.iter().take(row).sum::<f32>() + row as f32 * v_gap;

            let w = (0..cell.span)
                .map(|i| col_widths.get(col + i).copied().unwrap_or(0.0))
                .sum::<f32>()
                + h_gap * (cell.span.saturating_sub(1) as f32);

            let h = row_heights[row];

            cell.widget.arrange(Rect { x, y, w, h }, params, measurer);

            col += cell.span;
            if col >= cols { col = 0; row += 1; }
        }
    }

    fn collect_rects_inner(&self, out: &mut Vec<WidgetBase>) {
        out.push(self.base);
        for cell in self.container.cells() {
            collect_rects(cell.widget.as_ref(), out);
        }
    }

    fn collect_text_inner(&self, out: &mut Vec<TextParam>, params: &LayoutParams) {
        for cell in self.container.cells() {
            collect_text(cell.widget.as_ref(), out, params);
        }
    }
}
 