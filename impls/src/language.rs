pub mod display_text;

use base::prelude::Language;

/// An example of how to define your supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanadianLanguage {
    #[default]
    English,
    French,
}

impl Language for CanadianLanguage {
    fn id(&self) -> &'static str {
        match self {
            Self::English => "en-CA",
            Self::French => "fr-CA", // Don't forget to handle every variant!
        }
    }
}
