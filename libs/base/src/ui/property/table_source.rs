use std::ops::Range;

pub trait TableSource {
    type Row;

    fn total_rows(&self) -> usize;
    fn fetch_rows(&self, range: Range<usize>) -> Vec<&Self::Row>;
}

#[derive(Debug, Default)]
pub struct VecTableSource<T> {
    rows: Vec<T>,
}

impl<T> VecTableSource<T> {
    pub fn new(rows: Vec<T>) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    pub fn push(&mut self, row: T) {
        self.rows.push(row);
    }
}

impl<T> TableSource for VecTableSource<T> {
    type Row = T;

    fn total_rows(&self) -> usize {
        self.rows.len()
    }

    fn fetch_rows(&self, range: Range<usize>) -> Vec<&Self::Row> {
        let start = range.start.min(self.rows.len());
        let end = range.end.min(self.rows.len());
        self.rows[start..end].iter().collect()
    }
}