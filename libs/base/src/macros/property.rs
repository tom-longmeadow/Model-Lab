#[macro_export]
macro_rules! property_key {
    ($config:ty, $variant:ident) => {
        $crate::property::schema::PropertySchema::<$config>::hash_key(
            stringify!($variant)
        )
    };

    ($config:ty, $raw_str:expr) => {
        $crate::property::schema::PropertySchema::<$config>::hash_key($raw_str)
    };
}


