
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFont {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Light,
    LightItalic,
    Thin,
    ThinItalic,
    Medium,
    MediumItalic,
    SemiBold,
    SemiBoldItalic,
    ExtraBold,
    ExtraBoldItalic,
}

macro_rules! font {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../../assets/fonts/JetBrainsMono-2/fonts/ttf/",
            $name
        ))
    };
}

#[derive(Clone, Debug)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN:       FontWeight = FontWeight(100);
    pub const LIGHT:      FontWeight = FontWeight(300);
    pub const NORMAL:     FontWeight = FontWeight(400);
    pub const MEDIUM:     FontWeight = FontWeight(500);
    pub const SEMI_BOLD:  FontWeight = FontWeight(600);
    pub const BOLD:       FontWeight = FontWeight(700);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
}

#[derive(Clone, Debug)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl TextFont {
    pub fn all() -> &'static [TextFont] {
        &[
            TextFont::Regular,
            TextFont::Bold,
            TextFont::Italic,
            TextFont::BoldItalic,
            TextFont::Light,
            TextFont::LightItalic,
            TextFont::Thin,
            TextFont::ThinItalic,
            TextFont::Medium,
            TextFont::MediumItalic,
            TextFont::SemiBold,
            TextFont::SemiBoldItalic,
            TextFont::ExtraBold,
            TextFont::ExtraBoldItalic,
        ]
    }

    pub fn font_bytes(&self) -> &'static [u8] {
        match self {
            TextFont::Regular         => font!("JetBrainsMono-Regular.ttf"),
            TextFont::Bold            => font!("JetBrainsMono-Bold.ttf"),
            TextFont::Italic          => font!("JetBrainsMono-Italic.ttf"),
            TextFont::BoldItalic      => font!("JetBrainsMono-BoldItalic.ttf"),
            TextFont::Light           => font!("JetBrainsMono-Light.ttf"),
            TextFont::LightItalic     => font!("JetBrainsMono-LightItalic.ttf"),
            TextFont::Thin            => font!("JetBrainsMono-Thin.ttf"),
            TextFont::ThinItalic      => font!("JetBrainsMono-ThinItalic.ttf"),
            TextFont::Medium          => font!("JetBrainsMono-Medium.ttf"),
            TextFont::MediumItalic    => font!("JetBrainsMono-MediumItalic.ttf"),
            TextFont::SemiBold        => font!("JetBrainsMono-SemiBold.ttf"),
            TextFont::SemiBoldItalic  => font!("JetBrainsMono-SemiBoldItalic.ttf"),
            TextFont::ExtraBold       => font!("JetBrainsMono-ExtraBold.ttf"),
            TextFont::ExtraBoldItalic => font!("JetBrainsMono-ExtraBoldItalic.ttf"),
        }
    }

    pub fn family_name(&self) -> &'static str {
        "JetBrains Mono"
    }

    pub fn weight(&self) -> FontWeight {
        match self {
            TextFont::Thin | TextFont::ThinItalic           => FontWeight::THIN,
            TextFont::Light | TextFont::LightItalic         => FontWeight::LIGHT,
            TextFont::Regular | TextFont::Italic            => FontWeight::NORMAL,
            TextFont::Medium | TextFont::MediumItalic       => FontWeight::MEDIUM,
            TextFont::SemiBold | TextFont::SemiBoldItalic   => FontWeight::SEMI_BOLD,
            TextFont::Bold | TextFont::BoldItalic           => FontWeight::BOLD,
            TextFont::ExtraBold | TextFont::ExtraBoldItalic => FontWeight::EXTRA_BOLD,
        }
    }

    pub fn font_style(&self) -> FontStyle {
        match self {
            TextFont::Italic
            | TextFont::BoldItalic
            | TextFont::LightItalic
            | TextFont::ThinItalic
            | TextFont::MediumItalic
            | TextFont::SemiBoldItalic
            | TextFont::ExtraBoldItalic => FontStyle::Italic,
            _ => FontStyle::Normal,
        }
    }
}