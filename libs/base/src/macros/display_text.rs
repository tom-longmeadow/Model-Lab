#[macro_export]
macro_rules! display_text_macro {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            #[default]
            $default_variant:ident $(= $default_override:expr)?,
            $(
                $variant:ident $(= $override:expr)?
            ),* $(,)?
        }
    ) => {
        // 1. Generate the standard enum
        $(#[$meta])*
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            #[default]
            $default_variant,
            $( $variant ),*
        }

        // 2. Generate the compile-time static strings and helpers
        impl $name {
            /// A master array of strings in the exact order of the enum.
            pub const VARIANTS: &'static [&'static str] = &[
                // Handle the default variant
                display_text_macro!(@string $default_variant $(= $default_override)?),
                // Handle the rest
                $( display_text_macro!(@string $variant $(= $override)?) ),*
            ];

            /// Non-const runtime method for your translation traits
            pub fn default_text(&self) -> &'static str {
                Self::VARIANTS[*self as usize]
            }

            /// 🌟 Safe to call in const blocks to get the fallback string
            /// This removes the need for noisy array-indexing at the call site!
            pub const fn as_str(self) -> &'static str {
                Self::VARIANTS[self as usize]
            }
        }
    };

    // Internal helper to handle overrides vs standard stringify
    (@string $variant:ident) => { stringify!($variant) };
    (@string $variant:ident = $override:expr) => { $override };
}
