
use std::collections::HashMap;
use super::{DynamicCategory, UnitCategory}; 


pub struct UnitRegistry {
    categories: HashMap<String, Box<dyn DynamicCategory>>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn register<T: UnitCategory + Default + Send + Sync + 'static>(&mut self) {
        let cat = T::default(); 
        self.categories.insert(T::id().to_string(), Box::new(cat));
    }

    pub fn get(&self, id: &str) -> Option<&dyn DynamicCategory> {
        self.categories.get(id).map(|b| b.as_ref())
    }
}



// fn doit() {

//     #[derive(Default)]
//     pub struct SectionArea;

//     // 2. Implement the physics and metadata
//     impl UnitCategory for SectionArea {
//         type UnitType = LengthUnit; // Reuses the Length macro-generated enum
//         fn id() -> &'static str { "section_area" }
//         fn category_name() -> &'static str { "Section Area" }
//         fn kind() -> UnitKind { UnitKind::area() } // This is L^2
//         fn default_unit() -> LengthUnit { LengthUnit::Millimeter }
//     }

//     // 3. Register it
//     let mut registry = UnitRegistry::new();
//     registry.register::<SectionArea>();

//     let area_prop = Property {
//         name: "Cross Section Area".to_string(),
//         category_id: "section_area".to_string(), // Matches SectionArea::id()
//         value: PropertyValue::Measurement {
//             value: 150.0,
//             unit: "mm".to_string(), // The symbol
//         },
//         description: Some("Area of the beam profile".to_string()),
//         is_readonly: false,
//     };
// }

// fn process_measurement(prop: &Property, registry: &UnitRegistry) {
//     // 1. Get the Measurement data
//     if let PropertyValue::Measurement { value, unit } = &prop.value {
        
//         // 2. Use the category_id to find the DynamicCategory in the registry
//         if let Some(dyn_cat) = registry.get(&prop.category_id) {
            
//             // 3. The DynamicCategory (which is secretly a SectionArea) 
//             //    does the math without us needing to know the types at compile-time!
//             let si_value = dyn_cat.to_si(*value, unit);
            
//             println!("Property: {}", prop.name);
//             println!("Original: {} {}", value, unit);
//             println!("SI Base (m²): {}", si_value);
            
//             // 4. UI Helper: get all valid units for a dropdown
//             let options = dyn_cat.symbols();
//             println!("Available units for this field: {:?}", options);
            
//         } else {
//             println!("Error: Category '{}' not found in registry!", prop.category_id);
//         }
//     }
// }
