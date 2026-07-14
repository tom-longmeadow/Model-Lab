 

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextKind {
    Heading,
    SubHeading,
    #[default]
    Label,
    Caption,
    TextField,
}

#[derive(Clone, Debug, Default)]
pub struct WidgetText {
    text: String,
    kind: TextKind,
}

impl WidgetText {
    pub fn new(text: impl Into<String>, kind: TextKind) -> Self {
        Self { text: text.into(), kind }
    }


    pub fn text(&self) -> &str { &self.text }
    pub fn kind(&self) -> TextKind { self.kind }
    pub fn set_text(&mut self, text: impl Into<String>) { self.text = text.into(); }
    pub fn set_kind(&mut self, kind: TextKind) { self.kind = kind; }
}