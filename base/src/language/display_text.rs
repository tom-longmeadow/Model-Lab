use crate::language::Language;




pub enum DisplayText {
    Width,
}

impl DisplayText {
    pub fn translate(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Width, Language::English) => "Width",
            (Self::Width, Language::French) => "Largeur",
        }
    }
}