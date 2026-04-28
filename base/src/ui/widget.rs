use std::sync::atomic::{AtomicU64, Ordering};

use crate::ui::{
    layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer},
    text::params::TextParam,
};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlKind {
    #[default]
    Label,
    TextField,
    Button,
    Panel,
    Flow,
}

#[derive(Clone, Copy, Debug)]
pub struct WidgetBase {
    id:      WidgetId,
    visible: bool,
    kind:    ControlKind,
    rect:    Rect,
}

impl WidgetBase {
    pub fn new(kind: ControlKind) -> Self {
        Self {
            id: WidgetId::next(),
            visible: true,
            kind,
            rect: Rect::default(),
        }
    }

    pub fn id(&self) -> WidgetId { self.id }
    pub fn kind(&self) -> ControlKind { self.kind }
    pub fn is_visible(&self) -> bool { self.visible }
    pub fn rect(&self) -> Rect { self.rect }
    pub fn set_rect(&mut self, rect: Rect) { self.rect = rect; }
    pub fn set_visible(&mut self, visible: bool) { self.visible = visible; }
    pub fn hide(&mut self) { self.visible = false; }
    pub fn show(&mut self) { self.visible = true; }
}

pub trait Widget {
    fn measure(&mut self, available: Size, params: &LayoutParams, measurer: &mut dyn TextMeasurer) -> Size;
    fn arrange(&mut self, rect: Rect, params: &LayoutParams, measurer: &mut dyn TextMeasurer);
    fn base(&self) -> &WidgetBase;

    fn collect_rects_inner(&self, _out: &mut Vec<WidgetBase>) {}
    fn collect_text_inner(&self, _out: &mut Vec<TextParam>, _params: &LayoutParams) {}
}

pub fn collect_rects(widget: &dyn Widget, out: &mut Vec<WidgetBase>) {
    if widget.base().is_visible() {
        widget.collect_rects_inner(out);
    }
}

pub fn collect_text(widget: &dyn Widget, out: &mut Vec<TextParam>, params: &LayoutParams) {
    if widget.base().is_visible() {
        widget.collect_text_inner(out, params);
    }
}