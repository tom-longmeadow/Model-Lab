use base::prelude::*;
use impls::examples::model::{ExampleModelConfig, ExampleUnitCategory};

#[derive(Clone, Debug)]
pub struct TestPart {
    pub name: String,
    pub size_mm: f64,
    pub material: String,
    pub mass_kg: f64,
    pub tolerance_mm: f64,
    pub batch: String,
}

impl TestPart {
    pub fn new() -> Self {
        Self {
            name: "Engine Bolt".to_string(),
            size_mm: 24.0,
            material: "Steel".to_string(),
            mass_kg: 1.42,
            tolerance_mm: 0.05,
            batch: "A-104".to_string(),
        }
    }

    pub const KEY_NAME: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.name");
    pub const KEY_SIZE_MM: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.size_mm");
    pub const KEY_MATERIAL: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.material");
    pub const KEY_MASS_KG: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.mass_kg");
    pub const KEY_TOL_MM: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.tolerance_mm");
    pub const KEY_BATCH: u64 = PropertySchema::<ExampleModelConfig>::hash_key("part.batch");
}

impl Propertied<ExampleModelConfig> for TestPart {
    fn get_schema() -> PropertyNode<ExampleModelConfig> {
        PropertyNode::Group {
            name: PropertyName::new_str("Part"),
            children: vec![

                PropertyNode::new_text_str("Name", Self::KEY_NAME),
                PropertyNode::new_number_str("Size", ExampleUnitCategory::Length, Self::KEY_SIZE_MM),
                PropertyNode::new_text_str("Material", Self::KEY_MATERIAL),
                // PropertyNode::Group {
                //     name: PropertyName::new_str("General"),
                //     children: vec![
                //         PropertyNode::new_text_str("Name", Self::KEY_NAME),
                //         PropertyNode::new_number_str("Size", ExampleUnitCategory::LengthSmall, Self::KEY_SIZE_MM),
                //         PropertyNode::new_text_str("Material", Self::KEY_MATERIAL),
                //     ],
                // },
                PropertyNode::Group {
                    name: PropertyName::new_str("Manufacturing"),
                    children: vec![
                        // no Mass category in ExampleUnitCategory, so keep as plain number for now 
                        PropertyNode::new_number_str("Tolerance", ExampleUnitCategory::LengthSmall, Self::KEY_TOL_MM),
                        PropertyNode::new_text_str("Batch", Self::KEY_BATCH),
                    ],
                },
            ],
        }
    }

    fn get_value(&self, key: u64) -> Option<PropertyValue> {
        match key {
            Self::KEY_NAME => Some(PropertyValue::Text(self.name.clone())),
            Self::KEY_SIZE_MM => Some(PropertyValue::Number(self.size_mm)),
            Self::KEY_MATERIAL => Some(PropertyValue::Text(self.material.clone())),
            Self::KEY_MASS_KG => Some(PropertyValue::Number(self.mass_kg)),
            Self::KEY_TOL_MM => Some(PropertyValue::Number(self.tolerance_mm)),
            Self::KEY_BATCH => Some(PropertyValue::Text(self.batch.clone())),
            _ => None,
        }
    }

    fn set_value(&mut self, key: u64, value: PropertyValue) {
        match (key, value) {
            (Self::KEY_NAME, PropertyValue::Text(v)) => self.name = v,
            (Self::KEY_SIZE_MM, PropertyValue::Number(v)) => self.size_mm = v,
            (Self::KEY_MATERIAL, PropertyValue::Text(v)) => self.material = v,
            (Self::KEY_MASS_KG, PropertyValue::Number(v)) => self.mass_kg = v,
            (Self::KEY_TOL_MM, PropertyValue::Number(v)) => self.tolerance_mm = v,
            (Self::KEY_BATCH, PropertyValue::Text(v)) => self.batch = v,
            _ => {}
        }
    }
}