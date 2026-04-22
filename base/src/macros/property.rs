
#[macro_export]
macro_rules! property_key {
    ($config:ty, $variant:ident) => {
        $crate::property::schema::PropertySchema::<$config>::hash_key(
            $crate::language::display_text::DisplayText::$variant.as_str()
        )
    };

    ($config:ty, $raw_str:expr) => {
        $crate::property::schema::PropertySchema::<$config>::hash_key($raw_str)
    };
}