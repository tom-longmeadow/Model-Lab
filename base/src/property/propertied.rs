use crate::{language::display_text::DisplayText, model::ModelConfig, property::PropertyNode, };

 
pub trait Propertied<C: ModelConfig> {
    /// Returns the template for this type of object.
    /// Note: Returns the Tree, which is a Vec of nodes.
    fn get_template() -> Vec<PropertyNode<C>> where Self: Sized;

    fn as_any(&self) -> &dyn std::any::Any;

    fn instance_name(&self) -> DisplayText;
}


// pub trait Propertied<K: UnitCategory> {

//     fn get_tree() -> Vec<PropertyNode<K>> where Self: Sized;

//     fn as_any(&self) -> &dyn std::any::Any;

//     fn instance_name(&self) -> DisplayText;
// }