pub mod base_unit; 
pub mod kind;
pub mod category;
pub mod simple;
pub mod compound;
pub mod settings;
pub mod config;

pub use base_unit::*; 
pub use kind::*;
pub use category::*;
pub use simple::*;
pub use compound::*;
pub use settings::*;

use crate::prelude::UnitConfig;
 

pub struct UnitSystem<Config: UnitConfig> {
    pub file: Config::UnitSetting,
    pub display: Config::UnitSetting,
}

impl<Config: UnitConfig> UnitSystem<Config> {
    pub fn new(file: Config::UnitSetting, display: Config::UnitSetting) -> Self {
        Self { file, display }
    }

    pub fn convert(&self, value: f64, category: Config::UnitCategory, from: &Config::UnitSetting, to: &Config::UnitSetting) -> f64 {
        let from_kind = from.get(category);
        let to_kind = to.get(category);
        to_kind.from_base(from_kind.to_base(value))
    }

    pub fn file_to_display(&self, value: f64, category: Config::UnitCategory) -> f64 {
        self.convert(value, category, &self.file, &self.display)
    }

     pub fn display_to_file(&self, value: f64, category: Config::UnitCategory) -> f64 {
        self.convert(value, category, &self.display, &self.file)
    }
  
    pub fn symbol(&self, category: Config::UnitCategory) -> String {
        self.display.get(category).symbol()
    }

     
}