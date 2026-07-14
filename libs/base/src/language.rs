
pub trait Language: Clone + Copy + Default { 
    fn locale_code(&self) -> &'static str;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    #[default]
    EnUs,
    EnCa,
    FrFr,
}

impl Language for Locale {
    fn locale_code(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::EnCa => "en-CA",
            Self::FrFr => "fr-FR",  
        }
    }
}

pub trait DisplayLanguage: 'static + Clone + Copy {
    fn translate<L: Language>(&self, lang: L) -> String;
    fn default_text(&self) -> &'static str;
    fn id_label() -> Self;
}

