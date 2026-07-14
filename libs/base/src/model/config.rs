use super::{ComponentKind, ComponentRegistry, HasKind};

use crate::{
    prelude::PropertyConfig
};


 


pub trait ModelConfig: PropertyConfig + Sized {
    type Kind: ComponentKind;
    type Data: Clone + HasKind<Kind = Self::Kind>;

    type Registry: ComponentRegistry<
        Self,
        Data = Self::Data,
        Id = <Self::Kind as ComponentKind>::Id
    >;
}

// pub trait ModelConfig: PropertyConfig + Sized {
//     type Data: ComponentData<Self>;
//     type Kind: ComponentKind<Self::Data>;
//     type Id: ComponentId<Self::Kind>;

//     // The registry pulls it directly from the Data's Kind mapping
//     type Registry: ComponentRegistry<
//         Self, 
//         Id = Self::Id, 
//         Data = Self::Data
//     >;
// }


// pub trait ModelConfig: PropertyConfig + Sized {
    
//     type Data: ComponentData<Self>;
//     type Id1 = <<Self::Data as ComponentData<C>>::Kind as ComponentKind>::Id;
//     type Id2 = <<<Self::Data as ComponentData<C>>::Kind as ComponentKind>::Id;
//     //type Id: ComponentId = <<<<Self::Data as ComponentData<Self>>::Kind as ComponentKind>::Id>;
     
//     type Registry: ComponentRegistry<Self, Id = Self::Id, Data = Self::Data>;
// }