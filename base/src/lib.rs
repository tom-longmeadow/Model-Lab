pub use derive_more; 

#[macro_use]
pub mod macros;  

pub mod model;
pub mod property;
pub mod unit;
pub mod language;

pub mod prelude {
    // 🌟 1. Grab everything directly from the parent modules
    pub use crate::language::*;
    pub use crate::unit::{
        base_unit::*, category::*, kind::*, simple::*, compound::*, settings::*,
        UnitSystem,
    };

    pub use crate::property::{
        value::*, propertied::*, node::*, name::*, schema::*,   
    };

    pub use crate::model::{
        component::*, registry::*,    
    };

    // #[cfg(feature = "testing")]
    // pub use crate::model::test_model::*;

    // #[cfg(test)]
    // pub use crate::model::test_model::*;
  
    pub use crate::unit::config::UnitConfig as UnitConfig; 
    pub use crate::property::config::PropertyConfig; 
    pub use crate::model::config::ModelConfig as ModelConfig; 
    pub use crate::property::error::PropertyError as PropertyError;
    pub use crate::model::error::ModelError as ModelError;
    
    // 🌟 3. Include the macros in the prelude
    pub use crate::{
        enum_macro, 
        enum_index_macro,
        base_unit_macro,
        temperature_unit_macro,
        component_id_macro,
        component_id_primitive_macro
    };
}

// pub use derive_more; 



// pub mod model;
// pub mod property;
// pub mod unit;
// pub mod language;

// pub mod prelude {

//     pub use crate::language::*; 
//     pub use crate::unit::*; 
//     pub use crate::property::*;
//     pub use crate::model::*;


//     // Often useful to include the macros in the prelude
//     pub use crate::{
//         enum_macro, 
//         enum_index_macro,
//         base_unit_macro,
//         temperature_unit_macro,
//         component_id_macro,
//         component_id_primitive_macro
//     };
// }

