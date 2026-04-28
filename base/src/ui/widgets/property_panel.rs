use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::prelude::{Propertied, PropertyConfig, PropertyNode};
use crate::ui::layout::text_measurer::TextMeasurer;
use crate::ui::widget::WidgetId;
use crate::unit::UnitSystem;
use crate::ui::{
    container::WidgetContainer,
    layout::{layout_params::LayoutParams, rect::Rect, size::Size},
    text::{params::TextParam}, 
    widget::{ControlKind, Widget, WidgetBase, collect_rects, collect_text},
    widgets::{
        column::Column,
        label::Label,
        row::Row,
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
            let mut column = Column::new();
            Self::build_tree(&mut column, &T::get_schema(), &*locked, system, lang, 0);
            container.push(Box::new(column));
        }

        Self {
            obj,
            base: WidgetBase::new(ControlKind::Panel),
            container,
            _c: PhantomData,
        }
    }

    pub fn object(&self) -> Arc<Mutex<T>> { self.obj.clone() }

    pub fn rebuild(&mut self, system: &UnitSystem<C>, lang: C::Lang) {
        self.container.clear();
        let locked = self.obj.lock().unwrap();
        let mut column = Column::new();
        Self::build_tree(&mut column, &T::get_schema(), &*locked, system, lang, 0);
        self.container.push(Box::new(column));
    }

    fn build_tree(
        parent: &mut Column,
        node: &PropertyNode<C>,
        object: &T,
        system: &UnitSystem<C>,
        lang: C::Lang,
        depth: usize,
    ) {
        match node {
            PropertyNode::Group { name, children } => {
                let mut header = Row::new();
                header.push(Box::new(Label::new(name.label(lang))));
                parent.push(Box::new(header));

                let mut nested = Column::new();
                for child in children {
                    Self::build_tree(&mut nested, child, object, system, lang, depth + 1);
                }
                parent.push(Box::new(nested));
            }
            PropertyNode::Leaf(schema) => {
                let mut row = Row::new();

                row.push(Box::new(Label::new(schema.name.label(lang))));

                let value = schema.get_formatted_value(object, system);
                row.push(Box::new(TextField::new(value).with_placeholder("—")));

                if let Some(cat) = schema.unit {
                    let symbol = system.symbol(cat).to_string();
                    if !symbol.is_empty() {
                        row.push(Box::new(Label::new(symbol)));
                    }
                }

                parent.push(Box::new(row));
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


// use std::marker::PhantomData;
// use std::sync::Arc;
// use std::sync::Mutex;

// use crate::prelude::PropertyNode;
// use crate::ui::widgets::column::Column;
// use crate::unit::UnitSystem;
// use crate::{prelude::{Propertied, PropertyConfig}, ui::{ 
//     container::WidgetContainer, 
//     layout::{layout_params::LayoutParams, rect::Rect, size::Size, text_measurer::TextMeasurer}, 
//     macros::{impl_widget_base, impl_widget_container}, 
//     text::params::TextParam, widget::{ControlKind, Widget, WidgetBase, collect_rects, collect_text} 
    
// }};

// pub struct PropertyPanel<T, C: PropertyConfig>
// where
//     T: Propertied<C>,
// {
//     obj:       Arc<Mutex<T>>,
//     base:      WidgetBase,
//     container: WidgetContainer,
//     _c:        PhantomData<C>,
// }

// impl<T, C: PropertyConfig> PropertyPanel<T, C>
// where
//     T: Propertied<C>,
// {
//     pub fn new(obj: Arc<Mutex<T>>, system: &UnitSystem<C>, lang: C::Lang) -> Self {
//         let mut container = WidgetContainer::new();
//         {
//             let locked = obj.lock().unwrap();
//             let mut column = Column::new();
//             Self::build_tree(&mut column, &T::get_schema(), &*locked, system, lang, 0);
//             container.push(Box::new(column));
//         }

//         Self {
//             obj,
//             base: WidgetBase::new(ControlKind::Panel),
//             container,
//             _c: PhantomData,
//         }
//     }

//     pub fn object(&self) -> Arc<Mutex<T>> { self.obj.clone() }


//     fn build_tree<T: Propertied<C>>(
//         parent: &mut Column,
//         node: &PropertyNode<C>,
//         object: &T,
//         system: &UnitSystem<C>,
//         lang: C::Lang,
//         params: &LayoutParams,
//         depth: usize,
//     ) {
//         match node {
//             PropertyNode::Group { name, children } => {
//                 let mut group_header = Row::new();
//                 group_header.push(Box::new(Label::new(name.label(lang))));
//                 parent.push(Box::new(group_header));

//                 let mut nested = Column::new();
//                 nested.set_gap(params.gap);

//                 for child in children {
//                     Self::build_tree(&mut nested, child, object, system, lang, params, depth + 1);
//                 }

//                 parent.push(Box::new(nested));
//             }
//             PropertyNode::Leaf(schema) => {
//                 // Property row: label | value field | unit
//                 let mut row = Row::new();

//                 row.push(Box::new(Label::new(schema.name.label(lang))));

//                 let value = schema.get_formatted_value(object, system);
//                 row.push(Box::new(TextField::new(value).with_placeholder("—")));

//                 let unit = match schema.unit {
//                     Some(cat) => system.symbol(cat).to_string(),
//                     None => String::new(),
//                 };

//                 if !unit.is_empty() {
//                     row.push(Box::new(Label::new(unit)));
//                 }

//                 parent.push(Box::new(row));
//             }
//         }
//     } 


// }



// // use std::marker::PhantomData;

// // use crate::{
// //     property::{
// //         config::PropertyConfig,
// //         node::PropertyNode,
// //         propertied::Propertied,
// //     },
// //     ui::{
// //         layout::{
// //             layout_params::LayoutParams}, 
// //             widgets::{
// //                 column::Column, 
// //                 label::Label, 
// //                 row::Row, 
// //                 text_field::TextField
// //             }
// //     },
// //     unit::UnitSystem,
// // };

// // pub struct PropertyPanel<C: PropertyConfig> {
// //     column: Column,
// //     _c: PhantomData<C>,
// // }

// // impl<C: PropertyConfig> PropertyPanel<C> {
   
// //      pub fn new<T: Propertied<C>>(
// //         object: &T,
// //         system: &UnitSystem<C>,
// //         lang: C::Lang,
// //         params: &LayoutParams,
// //     ) -> Self {
// //         let mut column = Column::new();
// //         let schema = T::get_schema();
// //         Self::build_tree(&mut column, &schema, object, system, lang, params, 0);
// //         Self { column, _c: PhantomData }
// //     }
    

// //     pub fn into_column(self) -> Column {
// //         self.column
// //     }

// //     pub fn column(&self) -> &Column {
// //         &self.column
// //     }

// //     pub fn column_mut(&mut self) -> &mut Column {
// //         &mut self.column
// //     }

// //     fn build_tree<T: Propertied<C>>(
// //         parent: &mut Column,
// //         node: &PropertyNode<C>,
// //         object: &T,
// //         system: &UnitSystem<C>,
// //         lang: C::Lang,
// //         params: &LayoutParams,
// //         depth: usize,
// //     ) {
// //         match node {
// //             PropertyNode::Group { name, children } => {
// //                 let mut group_header = Row::new();
// //                 group_header.push(Box::new(Label::new(name.label(lang))));
// //                 parent.push(Box::new(group_header));

// //                 let mut nested = Column::new();
// //                 nested.set_gap(params.gap);

// //                 for child in children {
// //                     Self::build_tree(&mut nested, child, object, system, lang, params, depth + 1);
// //                 }

// //                 parent.push(Box::new(nested));
// //             }
// //             PropertyNode::Leaf(schema) => {
// //                 // Property row: label | value field | unit
// //                 let mut row = Row::new();

// //                 row.push(Box::new(Label::new(schema.name.label(lang))));

// //                 let value = schema.get_formatted_value(object, system);
// //                 row.push(Box::new(TextField::new(value).with_placeholder("—")));

// //                 let unit = match schema.unit {
// //                     Some(cat) => system.symbol(cat).to_string(),
// //                     None => String::new(),
// //                 };

// //                 if !unit.is_empty() {
// //                     row.push(Box::new(Label::new(unit)));
// //                 }

// //                 parent.push(Box::new(row));
// //             }
// //         }
// //     }
// // }

// // impl<C: PropertyConfig> std::fmt::Debug for PropertyPanel<C> {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         f.debug_struct("PropertyPanel").finish()
// //     }
// // }







// // pub struct PropertyPanel<C: PropertyConfig> {
// //     obj: Propertied<C>,
// //     base:      WidgetBase,       // ControlKind::Panel
// //     container: WidgetContainer,  // inner Column built from schema 
// // } 