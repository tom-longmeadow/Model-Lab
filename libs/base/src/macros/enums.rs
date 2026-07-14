
#[macro_export]
macro_rules! enum_macro {
    ($name:ident { $first_variant:ident, $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub enum $name {
            #[default]
            $first_variant,
            $($variant),*
        }

        impl $name {
            /// The first variant defined in the macro
            pub const DEFAULT: Self = Self::$first_variant;

            pub const ALL: &[Self] = &[
                Self::$first_variant,
                $(Self::$variant),*
            ];

            pub const COUNT: usize = Self::ALL.len();
        }
    };
}

#[macro_export]
macro_rules! enum_index_macro {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(usize)] // Ensures the enum uses usize-sized discriminants
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const ALL: &[Self] = &[
                $(Self::$variant),*
            ];

            pub const COUNT: usize = Self::ALL.len();

            /// Returns the index of the unit for array mapping
            pub const fn index(&self) -> usize {
                *self as usize
            }

            /// Helper to get a variant back from an index
            pub const fn from_index(index: usize) -> Option<Self> {
                if index < Self::COUNT {
                    Some(Self::ALL[index])
                } else {
                    None
                }
            }
        }
    };
}