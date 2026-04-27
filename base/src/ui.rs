pub mod text;

use std::marker::PhantomData;

use crate::{
    prelude::{
        Propertied, PropertyConfig, PropertyName, PropertyNode, 
        PropertySchema
    }, 
    unit::UnitSystem
};

 

pub struct Column<C: PropertyConfig> {
    pub schema: PropertySchema<C>,
    pub path: Vec<PropertyName<C>>,  // e.g. ["Point", "X"] for display breadcrumbs
}


impl<C: PropertyConfig> Column<C> {
    pub fn header(&self, lang: C::Lang) -> String {
        self.schema.name.label(lang)
    }

    pub fn unit_label(&self, system: &UnitSystem<C>) -> String {
        match self.schema.unit {
            Some(cat) => system.symbol(cat).to_string(),
            None => String::new(),
        }
    }

    pub fn value(&self, object: &impl Propertied<C>, system: &UnitSystem<C>) -> String {
        self.schema.get_formatted_value(object, system)
    }

    pub fn from_schema(node: &PropertyNode<C>) -> Vec<Self> {
        let mut columns = Vec::new();
        Self::flatten_recursive(node, &mut Vec::new(), &mut columns);
        columns
    }

    fn flatten_recursive(
        node: &PropertyNode<C>,
        path: &mut Vec<PropertyName<C>>,
        out: &mut Vec<Self>,
    ) {
        match node {
            PropertyNode::Group { name, children } => {
                path.push(name.clone());
                for child in children {
                    Self::flatten_recursive(child, path, out);
                }
                path.pop();
            }
            PropertyNode::Leaf(schema) => {
                out.push(Column {
                    schema: schema.clone(),
                    path: path.clone(),
                });
            }
        }
    }
}
  


pub struct PropertyPanelRow {
    pub label: String,
    pub value: String,
    pub unit:  String,
}


pub struct PropertyPanel<'a, C: PropertyConfig, T: Propertied<C>> {
    pub columns: Vec<Column<C>>,
    pub object: &'a T,
    pub system: &'a UnitSystem<C>,
    _c: PhantomData<C>,
}

impl<'a, C: PropertyConfig, T: Propertied<C>> PropertyPanel<'a, C, T> {
    pub fn new(object: &'a T, system: &'a UnitSystem<C>) -> Self {
        Self {
            columns: Column::from_schema(&T::get_schema()),
            object,
            system,
            _c: PhantomData,
        }
    }

    // pub fn rows(&self, lang: C::Lang) -> impl Iterator<Item = PropertyPanelRow> + '_ {
    //     self.columns.iter().map(move |col| PropertyPanelRow {
    //         label: col.header(lang),
    //         value: col.value(self.object, self.system),
    //         unit:  col.unit_label(self.system),
    //     })
    // }

    // pub fn rows(&self, lang: C::Lang) -> impl Iterator<Item = Result<PanelRow, PropertyError>> + '_ {
    //     self.columns.iter().map(move |col| {
    //         Ok(PanelRow {
    //             label: col.header(lang),
    //             value: col.schema.try_get_as_str(self.object, self.system)?,
    //             unit:  col.unit_label(self.system),
    //         })
    //     })
    // }
}


// // ── Horizontal (many objects, virtualized) ────────────────────────────────────

/// Trait your data source implements — keeps the sheet decoupled from storage
pub trait RowSource<C: PropertyConfig> {
    type Row: Propertied<C>;
    fn total_rows(&self) -> usize;
    fn fetch_rows(&self, range: std::ops::Range<usize>) -> Vec<&Self::Row>;
}

/// The visible window into the data
pub struct Viewport {
    pub row_offset: usize,     // first visible row index
    pub col_offset: usize,     // first visible column index  
    pub visible_rows: usize,   // how many rows fit on screen
    pub visible_cols: usize,   // how many cols fit on screen
    pub row_buffer:   usize,   // extra rows to fetch beyond visible (e.g. 50)
}

impl Viewport {
    pub fn fetch_range(&self) -> std::ops::Range<usize> {
        let start = self.row_offset.saturating_sub(self.row_buffer);
        let end   = self.row_offset + self.visible_rows + self.row_buffer;
        start..end
    }
}

pub struct PropertyGrid<'a, C: PropertyConfig, S: RowSource<C>> {
    pub all_columns:     Vec<Column<C>>,   // full schema, all properties
    pub visible_columns: Vec<usize>,       // indices into all_columns currently shown
    pub viewport:        Viewport,
    pub source:          &'a S,
    pub system:          &'a UnitSystem<C>,
    _c: PhantomData<C>,
}

impl<'a, C: PropertyConfig, S: RowSource<C>> PropertyGrid<'a, C, S> {
    // pub fn new(source: &'a S, system: &'a UnitSystem<C>, viewport: Viewport) -> Self
    //     where S::Row: Propertied<C> {
    // //     let all_columns = flatten_schema(&S::Row::get_schema());
    // //     let visible_columns = (0..all_columns.len()).collect();
    // //     Self { all_columns, visible_columns, viewport, source, system, _c: PhantomData }
    // }

    /// Column headers for currently visible columns
    pub fn headers(&self) -> Vec<String> {
        self.visible_columns.iter()
            .map(|&i| self.all_columns[i].schema.name.to_string())
            .collect()
    }

    /// Fetches buffered rows and formats the visible columns for each
    pub fn cells(&self) -> Vec<Vec<String>> {
        let rows = self.source.fetch_rows(self.viewport.fetch_range());
        rows.iter().map(|row| {
            self.visible_columns.iter().map(|&i| {
                self.all_columns[i].schema.get_formatted_value(*row, self.system)
            }).collect()
        }).collect()
    }

    pub fn scroll_to_row(&mut self, row: usize) {
        self.viewport.row_offset = row.min(self.source.total_rows().saturating_sub(1));
    }

    pub fn show_column(&mut self, col_index: usize) {
        if !self.visible_columns.contains(&col_index) {
            self.visible_columns.push(col_index);
            self.visible_columns.sort();
        }
    }

    pub fn hide_column(&mut self, col_index: usize) {
        self.visible_columns.retain(|&i| i != col_index);
    }
}