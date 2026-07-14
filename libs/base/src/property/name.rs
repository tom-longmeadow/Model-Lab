use std::fmt;
use crate::prelude::{DisplayLanguage, PropertyConfig};



#[derive(Debug, PartialEq, Eq)] 
pub enum PropertyName<C: PropertyConfig> {
    Text(C::Display),
    String(String),
}

impl<C: PropertyConfig> Clone for PropertyName<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Text(d)   => Self::Text(d.clone()),
            Self::String(s) => Self::String(s.clone()),
        }
    }
}


impl<C: PropertyConfig> fmt::Display for PropertyName<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(key) => write!(f, "{}", key.default_text()),
            Self::String(s) => write!(f, "{}", s),  
        }
    }
}

impl<C: PropertyConfig> PropertyName<C> {
    pub fn new(name: C::Display) -> Self {
        Self::Text(name)
    }

    pub fn new_str(name: impl Into<String>) -> Self {
        Self::String(name.into())
    } 

    pub fn label(&self, lang: C::Lang) -> String {
        match self {
            Self::Text(key) => key.translate(lang),
            Self::String(s) => s.clone(),
        }
    }
}
 

impl<C: PropertyConfig> From<&str> for PropertyName<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<C: PropertyConfig> From<String> for PropertyName<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
 