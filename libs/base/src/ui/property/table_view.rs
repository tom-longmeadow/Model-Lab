use std::ops::Range;

use crate::ui::property::table_source::TableSource;

pub struct TableView<'a, S: TableSource> {
    pub source: &'a S,
    pub viewport: TableViewport,
}

impl<'a, S: TableSource> TableView<'a, S> {
    pub fn total_rows(&self) -> usize {
        self.source.total_rows()
    }

    pub fn fetch_rows(&self) -> Vec<&S::Row> {
        self.source.fetch_rows(self.viewport.fetch_range())
    }

    pub fn scroll_to_row(&mut self, row: usize) {
        self.viewport.row_offset = row.min(self.total_rows().saturating_sub(1));
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TableViewport {
    pub row_offset: usize,
    pub col_offset: usize,
    pub visible_rows: usize,
    pub visible_cols: usize,
    pub row_buffer: usize,
}

impl TableViewport {
    pub fn fetch_range(&self) -> Range<usize> {
        let start = self.row_offset.saturating_sub(self.row_buffer);
        let end = self.row_offset + self.visible_rows + self.row_buffer;
        start..end
    }
}