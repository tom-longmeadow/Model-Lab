
#[macro_export]
macro_rules! base_unit_macro {
    ($name:ident { 
        $default_variant:ident = ($default_factor:expr, $default_symbol:expr), 
        $($variant:ident = ($factor:expr, $symbol:expr)),* $(,)? 
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
        pub enum $name {
            #[default]
            $default_variant,
            $($variant),*
        }

        impl Unit for $name {
            // This now matches the trait definition
            const COUNT: usize = 1 $(+ { let _ = stringify!($variant); 1 })*;

            fn symbol(&self) -> &'static str {
                match self {
                    Self::$default_variant => $default_symbol,
                    $(Self::$variant => $symbol),*
                }
            }

            fn all_variants() -> Vec<Self> {
                vec![Self::$default_variant, $(Self::$variant),*]
            }

            fn to_si(&self, val: f64, power: i8) -> f64 {
                val * self.conversion_factor().powi(power as i32)
            }

            fn from_si(&self, si_val: f64, power: i8) -> f64 {
                si_val / self.conversion_factor().powi(power as i32)
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
        
        // Bonus: Implementation for easy printing
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.symbol())
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
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
        pub enum $name {
            #[default]
            $default_variant,
            $($variant),*
        }

        impl Unit for $name {
            const COUNT: usize = 1 $(+ { let _ = stringify!($variant); 1 })*;

            fn symbol(&self) -> &'static str {
                match self {
                    Self::$default_variant => $default_symbol,
                    $(Self::$variant => $symbol),*
                }
            }
    
            fn all_variants() -> Vec<Self> {
                vec![Self::$default_variant, $(Self::$variant),*]
            }
     
            fn to_si(&self, val: f64, power: i8) -> f64 {
                let (factor, offset) = self.values();
                if power == 0 {
                    // It's a Temperature Delta (Difference)
                    val * factor
                } else {
                    // It's an Absolute Temperature
                    (val * factor) + offset
                }
            }

            fn from_si(&self, si_val: f64, power: i8) -> f64 {
                let (factor, offset) = self.values();
                if power == 0 {
                    si_val / factor
                } else {
                    (si_val - offset) / factor
                }
            }
        }

        impl $name {
            #[inline]
            fn values(&self) -> (f64, f64) {
                match self {
                    Self::$default_variant => ($default_factor, $default_offset),
                    $(Self::$variant => ($factor, $offset)),*
                }
            }
        }
    };
}