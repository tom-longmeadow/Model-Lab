use super::{BaseUnit};

/// Exponents to use for unit conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]  
pub struct UnitDimensions {
    pub exponents: [i8; BaseUnit::COUNT],
} 

impl UnitDimensions {
    /// Create an empty (dimensionless) set of exponents.
    pub const fn empty() -> Self {
        Self { exponents: [0; BaseUnit::COUNT] }
    }
    
    /// Update a specific exponent. Useful for building dimensions at runtime.
    pub fn set(&mut self, unit: BaseUnit, exponent: i8) {
        self.exponents[unit as usize] = exponent;
    }

    /// M^1 * L^1 * T^-2
    pub const fn force() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = 1;
        e[BaseUnit::Time as usize] = -2;
        Self { exponents: e }
    }

    /// M^1 * L^-1 * T^-2
    pub const fn stress() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = -1;
        e[BaseUnit::Time as usize] = -2;
        Self { exponents: e }
    }

    /// M^1 * L^2 * T^-2 (Same as Energy/Work)
    pub const fn torque() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = 2;
        e[BaseUnit::Time as usize] = -2;
        Self { exponents: e }
    }

    /// M^1 * L^1 * T^-1
    pub const fn linear_momentum() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = 1;
        e[BaseUnit::Time as usize] = -1;
        Self { exponents: e }
    }

    /// M^1 * L^-1 * T^-1 (Dynamic Viscosity, e.g., Pa·s)
    pub const fn dynamic_viscosity() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = -1;
        e[BaseUnit::Time as usize] = -1;
        Self { exponents: e }
    }

    /// L^2 * T^-1 (Kinematic Viscosity, e.g., m²/s)
    pub const fn kinematic_viscosity() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Length as usize] = 2;
        e[BaseUnit::Time as usize] = -1;
        Self { exponents: e }
    }

    /// M^1 * L^2 * T^-3 (Rate of doing work)
    pub const fn power() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Mass as usize] = 1;
        e[BaseUnit::Length as usize] = 2;
        e[BaseUnit::Time as usize] = -3;
        Self { exponents: e }
    }

    /// L^1 * T^-2 (Change in velocity over time)
    pub const fn acceleration() -> Self {
        let mut e = [0i8; BaseUnit::COUNT];
        e[BaseUnit::Length as usize] = 1;
        e[BaseUnit::Time as usize] = -2;
        Self { exponents: e }
    }
}


 


     

// pub trait UnitCategory {
//     type UnitType: Unit; 

//     fn id() -> &'static str;             // Added for serialization
//     fn mode() -> CategoryMode;
//     fn category_name() -> &'static str;
//     fn default_unit() -> Self::UnitType;
    
//     // Optional: Default decimal places for this type of measurement
//     fn default_precision() -> usize { 2 }

//     fn allowed_units() -> Vec<Self::UnitType> {
//         Self::UnitType::all_variants()
//     }

//     /// Helper to get the underlying physics vector
//     fn dimensions() -> Dimensions {
//         Self::mode().dimensions()
//     }

    
// }