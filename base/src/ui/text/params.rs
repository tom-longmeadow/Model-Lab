 
use crate::ui::text::{
    item::TextItem,
    style::TextStyle,
};

pub struct TextParam {
    pub style: TextStyle,
    pub items: Vec<TextItem>,
}

impl TextParam {
    pub fn new(style: TextStyle, items: Vec<TextItem>) -> Self {
        Self { style, items }
    }
}


pub struct TextParams {
    pub groups: Vec<TextParam>,
}

impl TextParams {
    pub fn new(groups: Vec<TextParam>) -> Self {
        Self { groups }
    }

    pub fn single(style: TextStyle, items: Vec<TextItem>) -> Self {
        Self::new(vec![TextParam::new(style, items)])
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