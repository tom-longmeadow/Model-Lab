

use super::{
    UnitCategory, UnitKind,  
};

 /// Maps a UnitCategory to the Units to use for the category
pub trait UnitSettings<C: UnitCategory>: Copy + Clone + PartialEq {
    fn default() -> Self;
    fn get(&self, category: C) -> UnitKind;
}
