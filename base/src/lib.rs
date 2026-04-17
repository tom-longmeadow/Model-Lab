#[macro_use]
pub mod macros;  

pub mod model;
pub mod property;
pub mod unit;
pub mod language;

pub mod prelude {

    pub use crate::unit::{
        base_unit::*,
        category::*,
        dimensions::*,
        kind::*,
        UnitSetting,
        UnitSettings,
    };
    
    pub use crate::property::{
        propertied::*,
        property_node::*,
        error::*,
        value::*,
        Property,
    };

    pub use crate::model::{
        component::*,
        registry::*,
        error::*,
        ModelConfig,
        Model,
    };
    
    // Often useful to include the macros in the prelude
    pub use crate::{enum_macro, base_unit_macro};
}

