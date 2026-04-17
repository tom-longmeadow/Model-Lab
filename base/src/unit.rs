pub mod base_unit;
pub mod dimensions;
pub mod kind;
pub mod category;
 
pub use base_unit::*;
pub use dimensions::*;
pub use kind::*;
pub use category::*; 

 
use crate::model::ModelConfig;
 
pub trait UnitSetting<C: UnitCategory> { 
    fn get_symbol(&self, category: C) -> &'static str;
    fn to_si(&self, category: C, value: f64) -> f64;
    fn from_si(&self, category: C, si_value: f64) -> f64;
}


pub struct UnitSettings<Config: ModelConfig> {
    pub file: Config::Setting,
    pub display: Config::Setting,
}

impl<Config: ModelConfig> UnitSettings<Config> {
    pub fn new(file: Config::Setting, display: Config::Setting) -> Self {
        Self { file, display }
    }
}




// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum MyUnitCategory {
//     Length,
//     LengthSmall,
//     Force,
// }

// impl UnitCategory for MyUnitCategory {}

// pub struct MyUnitSetting {
//     pub length: LengthUnit,
//     pub length_small: LengthUnit,
//     pub force: ForceUnit,
// }

// impl UnitSetting<UnitCategory> for MyUnitSetting {
//     fn get_symbol(&self, key: MyUnitKey) -> &'static str {
//         match key {
//             MyUnitKey::Length => self.length.symbol(),
//             MyUnitKey::LengthSmall => self.length_small.symbol(),
//             MyUnitKey::Force => self.force.symbol(),
//         }
//     }

//     fn to_si(&self, key: MyUnitKey, value: f64) -> f64 {
//         match key {
//             MyUnitKey::Length => self.length.to_si(value, 1),
//             MyUnitKey::LengthSmall => self.length_small.to_si(value, 1),
//             MyUnitKey::Force => self.force.to_si(value, 1),
//         }
//     }
// }

 


// pub struct Spreadsheet<'a, C: ModelConfig> {
//     pub tree: Vec<PropertyNode<C>>,
//     pub instances: &'a [Box<dyn Propertied<C>>], 
// }

// pub struct Spreadsheet<'a, K: UnitKey, P: Propertied<K>> {
//     pub tree: PropertyNode<K>,
//     pub instances: &'a [P],
// }


// impl<'a, K: UnitKey, P: Propertied<K>> Spreadsheet<'a, K, P> {
//     pub fn get_cell<S: UnitSetting<K>>(
//         &self, 
//         instance_index: usize, 
//         leaf: &Property<K>, 
//         settings: &S
//     ) -> String {
//         let instance = &self.instances[instance_index];
        
//         // 1. Extract the raw value
//         let val = (leaf.extractor)(instance);

//         // 2. Format based on variant and key
//         match (val, leaf.key) {
//             (PropertyValue::Number(n), Some(key)) => {
//                 let symbol = settings.get_symbol(key);
//                 format!("{:.2} {}", n, symbol)
//             }
//             (PropertyValue::Text(t), _) => t,
//             (PropertyValue::Number(n), None) => n.to_string(),
//             (PropertyValue::Integer(i), _) => i.to_string(),
//             (PropertyValue::Boolean(b), _) => b.to_string(),
//         }
//     }
// }



 

// pub struct Model<C: ModelConfig> {
//     pub registry: ComponentRegistry<Id = C::Id, Data = C::Data>,
//     pub settings: C::Setting,
// }


// impl<C: ModelConfig> PropertyTemplate<C> {
//     pub fn evaluate(&self, data: &C::Data) -> PropertyValue<C::Key> {
//         let raw = (self.extractor)(data);
        
//         match (raw, self.unit_key) {
//             (RawValue::Number(v), Some(key)) => PropertyValue::Measurement {
//                 value: v,
//                 unit_key: key,
//             },
//             (RawValue::Text(s), _) => PropertyValue::Text(s),
//             (RawValue::Boolean(b), _) => PropertyValue::Boolean(b),
//             (RawValue::Integer(i), _) => PropertyValue::Integer(i),
//             (RawValue::Number(v), None) => PropertyValue::Number(v),
//         }
//     }
// }

// impl<C: ModelConfig> TemplateNode<C> {
//     /// Recursively find all leaves to create spreadsheet columns
//     pub fn flatten(&self, path: Vec<String>, columns: &mut Vec<(Vec<String>, &PropertyTemplate<C>)>) {
//         match self {
//             TemplateNode::Leaf(template) => {
//                 columns.push((path, template));
//             }
//             TemplateNode::Group { name, children } => {
//                 let mut new_path = path.clone();
//                 new_path.push(name.clone());
//                 for child in children {
//                     child.flatten(new_path.clone(), columns);
//                 }
//             }
//         }
//     }
// }





// usage





// let width_column = PropertyTemplate {
//     name: DisplayTextEnum.Width,
//     unit_key: Some(MyUnitKey::LengthSmall),
//     extractor: |data| RawValue::Number(data.width), 
// };

// let location_group = TemplateNode::Group {
//     name: "Location".into(),
//     children: vec![
//         TemplateNode::Leaf(PropertyTemplate { name: "x".into(), unit_key: Some(MyUnitKey::Length), extractor: |d| RawValue::Number(d.x) }),
//         TemplateNode::Leaf(PropertyTemplate { name: "y".into(), unit_key: Some(MyUnitKey::Length), extractor: |d| RawValue::Number(d.y) }),
//         TemplateNode::Leaf(PropertyTemplate { name: "z".into(), unit_key: Some(MyUnitKey::Length), extractor: |d| RawValue::Number(d.z) }),
//     ]
// };

// let raw_val = (template.extractor)(component_data);
// if let (RawValue::Number(v), Some(key)) = (raw_val, template.unit_key) {
//     // The settings struct knows the category logic for this key!
//     let si_value = model.settings.to_si(key, v);
// }

// Vertical Properties with Group names:

// Joint
//     ID: 1
//     Location:
//         x: 1
//         y: 1
//         z = 23
//     DOF:
//         x: fixed
//         y: free
//         z: fixed

// Horizontally in a spreadsheets:
// Joint
//     Location    DOF
// ID  x   y   z   x       y       z
// 1   1   1   23  fixed   free    fixed
// 2   6   -8  0   free    free    free



// pub struct Property<K: UnitKey> {
//     pub name: String,
//     pub value: PropertyValue<K>,
//     pub category_id: String, 
//     pub description: Option<String>,
//     pub is_readonly: bool,
// }




// trait UnitKey {}

// trait UnitSetting<UnitKey> {
//     // A helper to resolve the key into a symbol
//     fn get_symbol(&self, key: MyUnitKey) -> &'static str;
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum MyUnitKey {
//     Length,
//     LengthSmall,
//     Force,
// }

// pub struct MyUnitSetting {
//     pub length: LengthUnit,
//     pub length_small: LengthUnit,
//     pub force: ForceUnit,
// }

// impl UnitSetting<UnitKey> for MyUnitSetting {
//     // A helper to resolve the key into a symbol
//     fn get_symbol(&self, key: MyUnitKey) -> &'static str {
//         match key {
//             MyUnitKey::Length => self.length.symbol(),
//             MyUnitKey::LengthSmall => self.length_small.symbol(),
//             MyUnitKey::Force => self.force.symbol(),
//         }
//     }
// }
 

// pub struct Model<I, D, R, k, U> 
// where 
//     I: ComponentId, 
//     D: ComponentData, 
//     R: ComponentRegistry<Id = I, Data = D>, 
//     K: UnitKey,
//     U: UnitSetting<K>
// {
//     registry: R, 
// }





// pub trait UnitSetting: Sized {
//     fn new() -> Self;
// }

// pub struct MyUnitSetting {
//     pub length: LengthUnit,
//     pub length_small: LengthUnit,
//     pub force: ForceUnit,
// }

// impl UnitSetting for MyUnitSetting {
//     fn new() -> Self {
//         Self {
//             length: LengthUnit::default(),
//             length_small: LengthUnit::default(),
//             force: ForceUnit::default(),
//         }
//     }
// }

// pub struct MyProperty {
//     pub name: String,
//     pub category_id: String,  
//     pub value: PropertyValue(MyUnitSetting::length), // Measurement { value: 10.0, unit: "mm2" }
//     pub description: Option<String>,
//     pub is_readonly: bool,
// }
 


//  trait TheUnitCategory {}



//  impl TheUnitCategory for MyUnitSettings{}



// pub struct UnitSettings<C>
//     where C: TheUnitCategory 
// {
//     // Maps the Category (e.g., Length) to the specific Unit (e.g., Meter)
//     selections: HashMap<MyUnitCategory, String>, 
// }

// impl UnitSettings {
//     pub fn get_unit(&self, category: MyUnitCategory) -> String {
//         self.selections.get(&category)
//             .cloned()
//             .unwrap_or_else(|| category.default_symbol())
//     }
// }

// // The user creates their "Active" settings
// let current_settings = vec![
//     MyUnitSettings::Length(LengthUnit::Meter),
//     MyUnitSettings::Deflection(LengthUnit::Millimeter),
// ];

 