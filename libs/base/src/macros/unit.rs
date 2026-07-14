

#[macro_export]
macro_rules! base_unit_macro {
    ($name:ident { 
        $default_variant:ident = ($default_factor:expr, $default_symbol:expr), 
        $($variant:ident = ($factor:expr, $symbol:expr)),* $(,)?
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $default_variant,
            $($variant),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$default_variant
            }
        }

        impl $name {
            pub fn conversion_factor(&self) -> f64 {
                match self {
                    Self::$default_variant => $default_factor,
                    $(Self::$variant => $factor),*
                }
            }
        }

        impl Unit for $name {
            const DEFAULT: Self = Self::$default_variant;
            const COUNT: usize = { 1 $(+ { let _ = stringify!($variant); 1 })* };

             

            fn symbol(&self) -> &'static str {
                match self {
                    Self::$default_variant => $default_symbol,
                    $(Self::$variant => $symbol),*
                }
            }

            fn all_variants() -> Vec<Self> {
                vec![Self::$default_variant, $(Self::$variant),*]
            }

            fn to_base(&self, val: f64, power: i8) -> f64 {
                val * self.conversion_factor().powi(power as i32)
            }

            fn from_base(&self, si_val: f64, power: i8) -> f64 {
                si_val / self.conversion_factor().powi(power as i32)
            }
        }
    };
}
#[macro_export]
macro_rules! temperature_unit_macro {
    ($name:ident { 
        $default_variant:ident = ($default_factor:expr, $default_offset:expr, $default_symbol:expr), 
        $($variant:ident = ($factor:expr, $offset:expr, $symbol:expr)),* $(,)? 
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $default_variant,
            $($variant),*
        }

        impl Default for $name {
            fn default() -> Self { Self::$default_variant }
        }

        impl $name {
            #[inline]
            fn values(&self) -> (f64, f64) {
                match self {
                    Self::$default_variant => ($default_factor, $default_offset),
                    $(Self::$variant => ($factor, $offset)),*
                }
            }

            /// Convert a temperature delta — factor only, no offset
            pub fn to_base_delta(&self, val: f64, power: i8) -> f64 {
                let (factor, _) = self.values();
                val * factor.powi(power as i32)
            }

            /// Convert a temperature delta from SI — factor only, no offset
            pub fn from_base_delta(&self, si_val: f64, power: i8) -> f64 {
                let (factor, _) = self.values();
                si_val / factor.powi(power as i32)
            }
        }

        impl Unit for $name {
            const DEFAULT: Self = Self::$default_variant;
            const COUNT: usize = { 1 $(+ { let _ = stringify!($variant); 1 })* };

            

            fn symbol(&self) -> &'static str {
                match self {
                    Self::$default_variant => $default_symbol,
                    $(Self::$variant => $symbol),*
                }
            }
    
            fn all_variants() -> Vec<Self> {
                vec![Self::$default_variant, $(Self::$variant),*]
            }
     
            fn to_base(&self, val: f64, power: i8) -> f64 {
                let (factor, offset) = self.values();
                if power == 0 { val * factor } else { (val * factor) + offset }
            }

            fn from_base(&self, si_val: f64, power: i8) -> f64 {
                let (factor, offset) = self.values();
                if power == 0 { si_val / factor } else { (si_val - offset) / factor }
            }
        }
    };
}

// #[macro_export]
// macro_rules! define_unit_settings {
//     ($category_enum:ident, $settings_struct:ident {
//         $($field:ident : $variant:ident : $kind:ident),* $(,)?
//     }) => {
//         #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//         pub enum $category_enum {
//             $($variant),*
//         }

//         impl UnitCategory for $category_enum {}

//         #[derive(Debug, Clone, Copy, PartialEq)]
//         pub struct $settings_struct {
//             // Appends "Unit" to the Kind for the type (e.g. Simple -> SimpleUnit)
//             $(pub $field: concat_idents!($kind, Unit)),*
//         }

//         impl UnitSettings<$category_enum> for $settings_struct {
//             // Note: Since defaults are highly specific (like mm vs m), 
//             // you usually have to implement 'fn default()' manually 
//             // or provide a default for each field.
            
//             fn get(&self, category: $category_enum) -> UnitKind {
//                 match category {
//                     $(
//                         $category_enum::$variant => {
//                             // Wraps field in UnitKind::Simple, UnitKind::Compound, etc.
//                             UnitKind::$kind(self.$field)
//                         }
//                     ),*
//                 }
//             }
//         }
//     };
// }