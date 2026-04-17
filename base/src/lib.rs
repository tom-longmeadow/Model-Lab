#[macro_use]
pub mod macros;  

pub mod model;
pub mod property;
pub mod unit;

pub mod prelude {

    pub use crate::unit::{
        base_unit::*,
        category::*,
        dimensions::*,
        kind::*,
    };
    
    pub use crate::property::{
        //component::*,
        //registry::*,
        error::*,
       // Model,
    };

    pub use crate::model::{
        component::*,
        registry::*,
        error::*,
        Model,
    };
    
    // Often useful to include the macros in the prelude
    pub use crate::{enum_macro, base_unit_macro};
}

