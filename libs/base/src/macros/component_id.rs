

#[macro_export]
macro_rules! component_id_primitive_macro {
    ($($t:ty),*) => {
        $(
            impl ComponentId for $t {
                fn invalid() -> Self { 0 }
            }
        )*
    };
}

#[macro_export]
macro_rules! component_id_macro {
    ($name:ident, $type:ty) => {

        #[repr(transparent)]
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        pub struct $name(pub $type);

        impl $name {
            pub fn new(val: $type) -> Option<Self> {
                let id = Self(val);
                if id.is_invalid() { None } else { Some(id) }
            }
        }

        impl ComponentId for $name {
            fn invalid() -> Self { Self(0) }
        }

        impl Default for $name {
            fn default() -> Self { Self::invalid() }
        }

        impl PartialEq<$type> for $name {
            fn eq(&self, other: &$type) -> bool { self.0 == *other }
        }

        impl std::ops::Deref for $name {
            type Target = $type;
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        // From / Into
        impl From<$type> for $name {
            fn from(val: $type) -> Self { Self(val) }
        }
        impl From<$name> for $type {
            fn from(id: $name) -> Self { id.0 }
        }

        // AsRef
        impl AsRef<$type> for $name {
            fn as_ref(&self) -> &$type { &self.0 }
        }

        // Display
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        // FromStr
        impl std::str::FromStr for $name {
            type Err = <$type as std::str::FromStr>::Err;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse::<$type>().map(Self)
            }
        }
    };
}

