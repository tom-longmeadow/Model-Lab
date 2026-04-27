use std::sync::atomic::{AtomicU64, Ordering};

use crate::ui::{
    text::params::TextParam, 
    widget::{r#box::BoxModel, layout::{
        rect::Rect, size::Size, text_measurer::TextMeasurer
    }}
};


pub mod widgets;
pub mod layout;
pub mod text;
pub mod container;
pub mod r#box;
pub mod macros;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

impl WidgetId {
    pub fn next() -> Self {
        Self(NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::next()
    }
}

 
pub trait Widget { 
    fn measure(&mut self, available: Size, measurer: &mut dyn TextMeasurer) -> Size;
    fn arrange(&mut self, rect: Rect, measurer: &mut dyn TextMeasurer); 
    fn collect_text(&self, _out: &mut Vec<TextParam>) {}
    fn collect_rects(&self, _out: &mut Vec<BoxModel>) {}
}


#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetBase {
    id: WidgetId,
    model: BoxModel,
}

impl WidgetBase {
    pub fn new() -> Self {
        Self {
            id: WidgetId::next(),
            model: BoxModel::default(),
        }
    }

    pub fn with_id(id: WidgetId) -> Self {
        Self {
            id,
            model: BoxModel::default(),
        }
    }


    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn box_model(&self) -> BoxModel {
        self.model
    }

    pub fn set_box_model(&mut self, model: BoxModel) {
        self.model = model;
    }
}
 
 
