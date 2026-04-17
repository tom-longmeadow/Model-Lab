use crate::{language::display_text::DisplayText, model::ModelConfig, property::Property};



pub enum PropertyNode<C: ModelConfig> {
    Leaf(Property<C>),
    Group {
        name: DisplayText,
        children: Vec<PropertyNode<C>>,
    },
}


// pub enum PropertyNode<K: UnitCategory> {
//     /// A data point (a column in a spreadsheet)
//     Leaf(Property<K>),
//     /// A logical grouping (a header in a spreadsheet)
//     Group {
//         name: DisplayText,
//         children: Vec<PropertyNode<K>>,
//     },
// }