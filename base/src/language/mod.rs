
 
pub trait Language: Clone + Copy + Default {
    fn id(&self) -> &'static str; // e.g., "en-CA", "es-ES"
}

pub trait TranslationProvider<L: Language>: 'static {
    /// Turns a DisplayKey into a localized string
    fn translate(&self, key: &dyn DisplayText, lang: L) -> String;
}

pub trait DisplayText: 'static {
    /// A fallback string if no translation is found
    fn default_text(&self) -> &'static str;
}


// #[derive(Default, Clone, Copy)]
// pub enum MyLanguage { #[default] English, Spanish }
// impl Language for MyLanguage {
//     fn id(&self) -> &'static str { match self { Self::English => "en", Self::Spanish => "es" } }
// }

// pub enum MyKeys { Width, Length }
// impl DisplayKey for MyKeys {
//     fn default_text(&self) -> &'static str { "Unknown" }
// }

// pub struct MyTranslator;
// impl TranslationProvider<MyLanguage> for MyTranslator {
//     fn translate(&self, key: &dyn DisplayKey, lang: MyLanguage) -> String {
//         // Here the user can use a match, a HashMap, or even load a JSON file
//         "Translated Text".to_string()
//     }
// }