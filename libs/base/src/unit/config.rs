 

use super::{
    UnitCategory,
    UnitSettings
};


pub trait UnitConfig: 'static {
    // Unit Categories
    type UnitCategory: UnitCategory + std::fmt::Debug + Clone + Copy + PartialEq;
    /// The Struct "Storage" that holds the actual SimpleUnits/CompoundUnits
    type UnitSetting: UnitSettings<Self::UnitCategory>;
}