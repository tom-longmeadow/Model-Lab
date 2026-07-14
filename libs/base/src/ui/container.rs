use std::fmt;

use crate::ui::widget::Widget;

pub trait Container {
    fn children(&self) -> &[Box<dyn Widget>];
    fn push(&mut self, child: Box<dyn Widget>);
    fn remove(&mut self, index: usize) -> Option<Box<dyn Widget>>;
    fn clear(&mut self);
}

pub struct WidgetContainer {
    children: Vec<Box<dyn Widget>>,
}

impl Default for WidgetContainer {
    fn default() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl fmt::Debug for WidgetContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetContainer")
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl WidgetContainer {
    pub fn new() -> Self { Self::default() }
    pub fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    pub fn push(&mut self, child: Box<dyn Widget>) { self.children.push(child); }
    pub fn push_children<I: IntoIterator<Item = Box<dyn Widget>>>(&mut self, children: I) {
        self.children.extend(children);
    }
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Widget>> {
        if index < self.children.len() { Some(self.children.remove(index)) } else { None }
    }
    pub fn clear(&mut self) { self.children.clear(); }
    pub(crate) fn for_each_child_mut(&mut self, mut f: impl FnMut(&mut Box<dyn Widget>)) {
        for child in &mut self.children { f(child); }
    }
}