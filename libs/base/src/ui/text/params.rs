 
use crate::ui::text::{
    item::TextItem,
    style::TextStyle,
};

pub struct TextGroup {
    pub style: TextStyle,
    pub items: Vec<TextItem>,
}

impl TextGroup {
    pub fn new(style: TextStyle, items: Vec<TextItem>) -> Self {
        Self { style, items }
    }
}


pub struct TextParams {
    pub groups: Vec<TextGroup>,
}

impl TextParams {
    pub fn new(groups: Vec<TextGroup>) -> Self {
        Self { groups }
    }

    pub fn single(style: TextStyle, items: Vec<TextItem>) -> Self {
        Self::new(vec![TextGroup::new(style, items)])
    }
}

impl Default for TextParams {
    fn default() -> Self {
        Self { groups: Vec::new() }
    }
}

// impl Default for TextParams {
//     fn default() -> Self {
//         Self::single(
//             TextStyle::default(),
//             vec![TextItem { text: "Hello".into(), x: 20.0, y: 20.0 }],
//         )
//     }
// }