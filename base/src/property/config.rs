use crate::prelude::{
        DisplayLanguage, DisplayText, Language, UnitConfig
    }; 

pub trait PropertyConfig: UnitConfig {
    type Display: DisplayLanguage + std::fmt::Debug + Clone + From<DisplayText> + Into<DisplayText>;  
    type Lang: Language; 
}
