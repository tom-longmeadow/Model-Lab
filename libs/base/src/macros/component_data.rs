#[macro_export]
macro_rules! component_data_macro {
    // number_fields only
    (
        $data:ty, $config:ty, $kind_type:ty, $kind_variant:expr,
        group: $group_name:expr,
        number_fields: { $( $field:ident : $key_ident:ident = $key_str:expr => $display:expr, $unit:expr ),* $(,)? }
    ) => {
        $crate::component_data_macro!($data, $config, $kind_type, $kind_variant,
            group: $group_name,
            number_fields: { $( $field: $key_ident = $key_str => $display, $unit ),* },
            id_fields: {}
        );
    };

    // id_fields only
    (
        $data:ty, $config:ty, $kind_type:ty, $kind_variant:expr,
        group: $group_name:expr,
        id_fields: { $( $id_field:ident : $id_key_ident:ident = $id_key_str:expr => $id_display:expr ),* $(,)? }
    ) => {
        $crate::component_data_macro!($data, $config, $kind_type, $kind_variant,
            group: $group_name,
            number_fields: {},
            id_fields: { $( $id_field: $id_key_ident = $id_key_str => $id_display ),* }
        );
    };

    // Full form
    (
        $data:ty, $config:ty, $kind_type:ty, $kind_variant:expr,
        group: $group_name:expr,
        number_fields: { $( $field:ident : $key_ident:ident = $key_str:expr => $display:expr, $unit:expr ),* $(,)? },
        id_fields: { $( $id_field:ident : $id_key_ident:ident = $id_key_str:expr => $id_display:expr ),* $(,)? }
    ) => {
        impl $data {
            $( pub const $key_ident: u64 = $crate::property::schema::PropertySchema::<$config>::hash_key($key_str); )*
            $( pub const $id_key_ident: u64 = $crate::property::schema::PropertySchema::<$config>::hash_key($id_key_str); )*
        }

        impl $crate::model::component::HasKind for $data {
            type Kind = $kind_type;
            fn kind(&self) -> $kind_type { $kind_variant }
        }

        impl $crate::model::component::ComponentData<$config> for $data {
            fn kind_name() -> $crate::property::name::PropertyName<$config> {
                $crate::property::name::PropertyName::new($group_name)
            }
        }

        impl $crate::property::propertied::Propertied<$config> for $data {
            fn get_schema() -> $crate::property::node::PropertyNode<$config> {
                $crate::property::node::PropertyNode::new_group($group_name, vec![
                    $( $crate::property::node::PropertyNode::new_number($display, $unit, <$data>::$key_ident), )*
                    $( $crate::property::node::PropertyNode::new_id($id_display, <$data>::$id_key_ident), )*
                ])
            }
            fn get_value(&self, key: u64) -> Option<$crate::property::value::PropertyValue> {
                match key {
                    $( <$data>::$key_ident => Some($crate::property::value::PropertyValue::Number(self.$field)), )*
                    $( <$data>::$id_key_ident => Some($crate::property::value::PropertyValue::ID(self.$id_field.to_string())), )*
                    _ => None,
                }
            }
            fn set_value(&mut self, key: u64, value: $crate::property::value::PropertyValue) {
                match key {
                    $( <$data>::$key_ident => { if let $crate::property::value::PropertyValue::Number(n) = value { self.$field = n; } } )*
                    $( <$data>::$id_key_ident => { if let $crate::property::value::PropertyValue::ID(s) = value { if let Ok(p) = s.parse() { self.$id_field = p; } } } )*
                    _ => (),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! model_config_macro {
    (
        $config:ty, $kind_type:ty, $data_enum:ty, $registry:ident,
        $unit_settings:ty,
        [ $( $variant:ident($inner:ty) => $kind_val:expr ),* $(,)? ]
    ) => {
        impl $crate::model::component::HasKind for $data_enum {
            type Kind = $kind_type;
            fn kind(&self) -> $kind_type {
                match self {
                    $( Self::$variant(_) => $kind_val, )*
                }
            }
        }

        impl $crate::model::config::ModelConfig for $config {
            type Kind = $kind_type;
            type Data = $data_enum;
            type Registry = $registry<$config>;
        }

        pub type ModelAlias = $crate::model::Model<$config>;

        pub fn get_model() -> ModelAlias {
            ModelAlias::new(
                $registry::new(),
                $crate::unit::UnitSystem::new(<$unit_settings>::default()),
            )
        }
    };
}