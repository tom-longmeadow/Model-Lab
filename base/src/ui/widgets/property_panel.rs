use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::prelude::{Propertied, PropertyConfig, PropertyNode};
use crate::ui::layout::text_measurer::TextMeasurer;
use crate::ui::widget::WidgetId;
use crate::ui::widget_text::TextKind;
use crate::unit::UnitSystem;
use crate::ui::{
    container::WidgetContainer,
    layout::{layout_params::LayoutParams, rect::Rect, size::Size},
    text::params::TextParam,
    widget::{ControlKind, Widget, WidgetBase, collect_rects, collect_text},
    widgets::{
        grid::Grid,
        label::Label,
        text_field::TextField,
    },
};

pub struct PropertyPanel<T, C: PropertyConfig>
where
    T: Propertied<C>,
{
    obj:       Arc<Mutex<T>>,
    base:      WidgetBase,
    container: WidgetContainer,
    _c:        PhantomData<C>,
}

impl<T, C: PropertyConfig> PropertyPanel<T, C>
where
    T: Propertied<C>,
{
    pub fn new(obj: Arc<Mutex<T>>, system: &UnitSystem<C>, lang: C::Lang) -> Self {
        let mut container = WidgetContainer::new();
        {
            let locked = obj.lock().unwrap();
            let mut grid = Grid::new(3);

            match T::get_schema() {
                PropertyNode::Group { name, children } => {
                    // root group name as Heading
                    grid.push_spanning(
                        Box::new(Label::with_kind(name.label(lang), TextKind::Heading)),
                        3,
                    );
                    for child in &children {
                        Self::build_into_grid(&mut grid, child, &*locked, system, lang);
                    }
                }
                node => Self::build_into_grid(&mut grid, &node, &*locked, system, lang),
            }
            container.push(Box::new(grid));
        }

        Self {
            obj,
            base: WidgetBase::new(ControlKind::Panel),
            container,
            _c: PhantomData,
        }
    }

    pub fn object(&self) -> Arc<Mutex<T>> { self.obj.clone() }

    fn build_into_grid(
        grid: &mut Grid,
        node: &PropertyNode<C>,
        object: &T,
        system: &UnitSystem<C>,
        lang: C::Lang,
    ) {
        match node {
            PropertyNode::Group { name, children } => {
                grid.push_spanning(
                    Box::new(Label::with_kind(name.label(lang), TextKind::SubHeading)),
                    3,
                );
                for child in children {
                    Self::build_into_grid(grid, child, object, system, lang);
                }
            }
            PropertyNode::Leaf(schema) => {
                let value = schema.get_formatted_value(object, system);
                let unit  = schema.unit
                    .map(|cat| system.symbol(cat).to_string())
                    .unwrap_or_default();

                grid.push_spanning(Box::new(Label::new(schema.name.label(lang))), 1);

                if unit.is_empty() {
                    grid.push_spanning(Box::new(TextField::new(value).with_placeholder("—")), 2);
                } else {
                    grid.push_spanning(Box::new(TextField::new(value).with_placeholder("—")), 1);
                    grid.push_spanning(Box::new(Label::new(unit)), 1);
                }
            }
        }
    }

    pub fn id(&self) -> WidgetId { self.base.id() }
    pub fn is_visible(&self) -> bool { self.base.is_visible() }
    pub fn set_visible(&mut self, visible: bool) { self.base.set_visible(visible); }
    pub fn hide(&mut self) { self.base.hide(); }
    pub fn show(&mut self) { self.base.show(); }
    pub fn rect(&self) -> Rect { self.base.rect() }
    pub fn set_rect(&mut self, rect: Rect) { self.base.set_rect(rect); }
    pub fn children(&self) -> &[Box<dyn Widget>] { self.container.children() }
    pub fn push(&mut self, child: Box<dyn Widget>) { self.container.push(child); }
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Widget>> { self.container.remove(index) }
    pub fn clear(&mut self) { self.container.clear(); }
}

impl<T, C: PropertyConfig> Widget for PropertyPanel<T, C>
where
    T: Propertied<C>,
{
    fn measure(&mut self, available: Size, params: &LayoutParams, measurer: &mut dyn TextMeasurer) -> Size {
        let padding = params.control.style_for(self.base.kind()).padding;
        let inner   = available.shrink(padding);
        let mut content_size = Size::zero();
        self.container.for_each_child_mut(|child| {
            content_size = child.measure(inner, params, measurer);
        });
        content_size.grow(padding)
    }

    fn arrange(&mut self, rect: Rect, params: &LayoutParams, measurer: &mut dyn TextMeasurer) {
        self.base.set_rect(rect);
        let padding = params.control.style_for(self.base.kind()).padding;
        let inner   = rect.inset(padding);
        self.container.for_each_child_mut(|child| child.arrange(inner, params, measurer));
    }

    fn base(&self) -> &WidgetBase { &self.base }

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

impl<T, C: PropertyConfig> std::fmt::Debug for PropertyPanel<T, C>
where
    T: Propertied<C>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyPanel").finish()
    }
}