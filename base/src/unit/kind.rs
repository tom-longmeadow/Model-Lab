
use super::{BaseUnit, UnitDimensions};

/// Type of unit.  Some units are m^2 (Simple) and some are kg*m/s^2 (Compound)
pub enum UnitKind {
    Simple(BaseUnit, i8),
    Compound(UnitDimensions),
    Temperature,
}


impl UnitKind {

    // --- Simple (BaseUnit Powers) ---
    pub const fn length() -> Self { Self::Simple(BaseUnit::Length, 1) }
    pub const fn area() -> Self { Self::Simple(BaseUnit::Length, 2) }
    pub const fn volume() -> Self { Self::Simple(BaseUnit::Length, 3) }
    pub const fn quartic() -> Self { Self::Simple(BaseUnit::Length, 4) }

    // --- Compound (Dimensions) ---
    pub const fn force() -> Self { Self::Compound(UnitDimensions::force()) }
    pub const fn stress() -> Self { Self::Compound(UnitDimensions::stress()) }
    pub const fn torque() -> Self { Self::Compound(UnitDimensions::torque()) }
    pub const fn power() -> Self { Self::Compound(UnitDimensions::power()) }
    pub const fn acceleration() -> Self { Self::Compound(UnitDimensions::acceleration()) }
    pub const fn dynamic_viscosity() -> Self { Self::Compound(UnitDimensions::dynamic_viscosity()) }
    pub const fn kinematic_viscosity() -> Self { Self::Compound(UnitDimensions::kinematic_viscosity()) }

    // --- Temperature ---
    pub const fn temperature() -> Self { Self::Temperature }

    pub const fn dimensions(&self) -> UnitDimensions {
        match self {
            Self::Simple(base, pow) => {
                let mut exponents = [0i8; BaseUnit::COUNT];
                exponents[*base as usize] = *pow;
                UnitDimensions { exponents }
            }
            Self::Compound(d) => *d,
            Self::Temperature => {
                let mut exponents = [0i8; BaseUnit::COUNT];
                exponents[BaseUnit::Temperature as usize] = 1;
                UnitDimensions { exponents }
            }
        }
    }

    
}