use crate::unit::UnitSystem;

use super::{
    config::PropertyConfig,
    name::PropertyName,
    propertied::Propertied,
    schema::PropertySchema,
    value::PropertyValueKind,
};

#[derive(Debug, Clone)]
pub enum PropertyNode<C: PropertyConfig> { 
    Group {
        name: PropertyName<C>,
        children: Vec<PropertyNode<C>>,
    }, 
    Leaf(PropertySchema<C>),
}

impl<C: PropertyConfig> PropertyNode<C> {
     
    pub fn name(&self) -> &PropertyName<C> {
        match self {
            Self::Group { name, .. } => name,
            Self::Leaf(schema) => &schema.name,
        }
    }

    /// Helper to get the pre-computed hash key of this node
    pub fn key(&self) -> u64 {
        match self {
            Self::Group { name, .. } => { 
                PropertySchema::<C>::hash_key(&name.to_string())
            }
            Self::Leaf(schema) => schema.key,
        }
    }

    pub fn new(
        name: C::Display,
        kind: PropertyValueKind,
        unit: Option<C::UnitCategory>,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new(name, kind, unit, key))
    }

    pub fn new_readonly(
        name: C::Display,
        kind: PropertyValueKind,
        unit: Option<C::UnitCategory>,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_readonly(name, kind, unit, key))
    }

    pub fn new_number(
        name: C::Display,
        unit: C::UnitCategory,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_number(name, unit, key))
    }

    pub fn new_text(
        name: C::Display, 
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_text(name, key))
    }

    pub fn new_id(
        name: C::Display,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_id(name, key))
    }

    pub fn new_id_readonly(
        name: C::Display,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_id_readonly(name, key))
    }


     pub fn new_str(
        name: impl Into<String>,
        kind: PropertyValueKind,
        unit: Option<C::UnitCategory>,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_str(name, kind, unit, key))
    }

    pub fn new_number_str(
         name: impl Into<String>,
        unit: C::UnitCategory,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_number_str(name, unit, key))
    }


    pub fn new_text_str(
        name: impl Into<String>,
        key: u64,
    ) -> Self {
        Self::Leaf(PropertySchema::new_text_str(name, key))
    }
  

    pub fn new_group(name: C::Display, children: Vec<PropertyNode<C>>) -> Self {
        Self::Group {
            name: PropertyName::new(name),
            children,
        }
    }

    pub fn flatten(&self) -> Vec<FlattenedProperty<C>> {
        let mut out = Vec::new();
        self.flatten_into(&mut Vec::new(), &mut out);
        out
    }

    pub fn visit_leaves(
        &self,
        f: &mut impl FnMut(&[PropertyName<C>], &PropertySchema<C>),
    ) {
        self.visit_leaves_inner(&mut Vec::new(), f);
    }

    fn flatten_into(
        &self,
        path: &mut Vec<PropertyName<C>>,
        out: &mut Vec<FlattenedProperty<C>>,
    ) {
        self.visit_leaves_inner(path, &mut |path, schema| {
            out.push(FlattenedProperty {
                schema: schema.clone(),
                path: path.to_vec(),
            });
        });
    }

    fn visit_leaves_inner(
        &self,
        path: &mut Vec<PropertyName<C>>,
        f: &mut impl FnMut(&[PropertyName<C>], &PropertySchema<C>),
    ) {
        match self {
            PropertyNode::Group { name, children } => {
                path.push(name.clone());
                for child in children {
                    child.visit_leaves_inner(path, f);
                }
                path.pop();
            }
            PropertyNode::Leaf(schema) => {
                f(path, schema);
            }
        }
    }
}
 
 

#[derive(Clone, Debug)]
pub struct FlattenedProperty<C: PropertyConfig> {
    pub schema: PropertySchema<C>,
    pub path: Vec<PropertyName<C>>,
}

impl<C: PropertyConfig> FlattenedProperty<C> {
    pub fn header(&self, lang: C::Lang) -> String {
        self.schema.name.label(lang)
    }

    pub fn unit_label(&self, system: &UnitSystem<C>) -> String {
        match self.schema.unit {
            Some(cat) => system.symbol(cat).to_string(),
            None => String::new(),
        }
    }

    pub fn value(&self, object: &impl Propertied<C>, system: &UnitSystem<C>) -> String {
        self.schema.get_formatted_value(object, system)
    }
}
 

 


 
    