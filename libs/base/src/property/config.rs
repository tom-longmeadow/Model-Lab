use crate::prelude::{
        DisplayLanguage, Language, UnitConfig
    }; 

// pub trait PropertyConfig: UnitConfig {
//     type Display: DisplayLanguage + std::fmt::Debug + Clone + From<DisplayText> + Into<DisplayText>;  
//     type Lang: Language; 
// }

pub trait PropertyConfig: UnitConfig {
    type Display: DisplayLanguage + std::fmt::Debug + Clone;  // removed From/Into DisplayText
    type Lang: Language; 
}