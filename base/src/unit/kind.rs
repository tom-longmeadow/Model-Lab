 
use super::{
    CompoundUnit, SimpleUnit, TemperatureUnit, Unit
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitKind {
    Simple(SimpleUnit),
    Compound(CompoundUnit),
    Temperature(TemperatureUnit),
}

impl UnitKind { 

    pub fn to_base(&self, val: f64) -> f64 {
        match self {
            Self::Simple(s) => s.to_base(val),
            Self::Temperature(t) => t.to_base(val, 1),
            Self::Compound(c) => { 
                let mut current_val = val;
                for comp in c.components.iter() {
                    if comp.exponent() != 0 { 
                        current_val = comp.to_base(current_val);
                    }
                }
                current_val
            }
        }
    }

    pub fn from_base(&self, val: f64) -> f64 {
        match self {
            Self::Simple(s) => s.from_base(val),
            Self::Temperature(t) => t.from_base(val, 1),
            Self::Compound(c) => {
                let mut current_val = val;
                for comp in c.components.iter() {
                    if comp.exponent() != 0 { 
                        current_val = comp.from_base(current_val);
                    }
                }
                current_val
            }
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Self::Simple(s) => {
                let exp = s.exponent();
                if exp == 1 { s.symbol().to_string() } 
                else { format!("{}^{}", s.symbol(), exp) }
            }
            Self::Temperature(t) => t.symbol().to_string(),
            Self::Compound(c) => {
                let parts: Vec<String> = c.components.iter()
                    .filter(|comp| comp.exponent() != 0)
                    .map(|comp| {
                        let exp = comp.exponent();
                        if exp == 1 { comp.symbol().to_string() } 
                        else { format!("{}^{}", comp.symbol(), exp) }
                    })
                    .collect();

                if parts.is_empty() { "unitless".to_string() } 
                else { parts.join("·") }
            }
        }
    }


    // pub fn to_base(&self, val: f64) -> f64 {
    //     match self {
    //         Self::Simple(s) => s.to_base(val),
    //         Self::Temperature(t) => t.to_base(val, 1),
    //         Self::Compound(c) => {
    //             let mut total_factor = 1.0;
    //             for comp in c.components {
    //                 if comp.exponent() != 0 {
    //                     // We need the raw conversion factor of the unit (e.g., 0.001 for mm)
    //                     // then raise it to the exponent.
    //                     // to_base(1.0) gives us that factor.
    //                     total_factor *= comp.to_base(1.0);
    //                 }
    //             }
    //             val * total_factor
    //         }
    //     }
    // }

    // pub fn from_base(&self, val: f64) -> f64 {
    //     match self {
    //         Self::Simple(s) => s.from_base(val),
    //         Self::Temperature(t) => t.from_base(val, 1),
    //         Self::Compound(c) => {
    //             let mut total_factor = 1.0;
    //             for comp in c.components {
    //                 if comp.exponent() != 0 {
    //                     total_factor *= comp.to_base(1.0);
    //                 }
    //             }
    //             val / total_factor
    //         }
    //     }
    // }

    // pub fn symbol(&self) -> String {
    //     match self {
    //         Self::Simple(s) => {
    //             let exp = s.exponent();
    //             if exp == 1 { s.symbol().to_string() } 
    //             else { format!("{}^{}", s.symbol(), exp) }
    //         }
    //         Self::Temperature(t) => t.symbol().to_string(),
    //         Self::Compound(c) => {
    //             let mut parts = Vec::new();
    //             for comp in c.components {
    //                 let exp = comp.exponent();
    //                 if exp != 0 {
    //                     if exp == 1 { parts.push(comp.symbol().to_string()); } 
    //                     else { parts.push(format!("{}^{}", comp.symbol(), exp)); }
    //                 }
    //             }
    //             if parts.is_empty() { "unitless".to_string() } 
    //             else { parts.join("·") }
    //         }
    //     }
    // }
 

}
 