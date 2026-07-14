use crate::ui::{ 
    layout::size::Size, text::style::TextStyle
};
 

pub trait TextMeasurer {
    fn measure(&mut self, text: &str, style: &TextStyle) -> Size;
}