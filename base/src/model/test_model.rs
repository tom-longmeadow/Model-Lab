
#[macro_export]
#[doc(hidden)]
macro_rules! __define_test_mocks {
    () => { 
        use super::*;
        use $crate::model::registry::ComponentRegistry;
        use $crate::model::component::{ComponentId, ComponentData, ComponentKind};
        use $crate::model::{Model, ModelError, ModelConfig};
        use $crate::unit::{UnitSettings, UnitSetting, UnitCategory};
        use $crate::language::{Language, DisplayText};



            // 1. Mock i18n Logic for the Test
        #[derive(Default, Clone, Copy)]
        pub enum MockLang { #[default] En }
        impl Language for MockLang {
            fn id(&self) -> &'static str { "en" }
        }

        #[derive(Clone, Copy, Default)]
        pub enum MockDisplayText { 
            #[default]
            Generic 
        }

        impl DisplayText for MockDisplayText {
            fn default_text(&self) -> &'static str { 
                "Generic" 
            }
        
            fn translate<L: Language>(&self, _lang: L) -> String {
                self.default_text().to_string()
            }
        }
    

        // 2. Mock Unit Logic
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum MockUnitCategory { #[default] Generic }
        impl UnitCategory for MockUnitCategory {}

        #[derive(Default, Clone, Copy)]
        pub struct MockUnitSetting;
        impl UnitSetting<MockUnitCategory> for MockUnitSetting {
            fn get_symbol(&self, _: MockUnitCategory) -> &'static str { "" }
            fn to_si(&self, _: MockUnitCategory, v: f64) -> f64 { v }
            fn from_si(&self, _: MockUnitCategory, v: f64) -> f64 { v }
        }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct MockId(pub u64);
        impl ComponentId for MockId {
            fn invalid() -> Self {
                Self(0)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum MockKind { // Added pub
            Kind1, 
            Kind2, 
        }
        impl ComponentKind for MockKind {}

        #[derive(Debug, Clone, PartialEq)]
        pub enum MockData { // Added pub
            Kind1{state: f32}, 
            Kind2{other: u32}, 
        }

        impl ComponentData for MockData {
            type Kind = MockKind;
            
            fn kind(&self) -> Self::Kind {
                match self {
                    Self::Kind1 { .. } => MockKind::Kind1,
                    Self::Kind2 { .. } => MockKind::Kind2,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! test_registry {
    ($registry_type:ident) => {
        #[cfg(test)]
        mod registry_tests {
            
            $crate::__define_test_mocks!(); 
            type TestRegistry = $registry_type<MockId, MockData>;

              
            #[test]
            fn test_raw_storage() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(8);
                let data_v1 = MockData::Kind1 { state: 10.0 };
                let data_v2 = MockData::Kind1 { state: 20.0 };

                // 1. First insert: Key is new, so it returns None
                let a = reg.insert(id1, data_v1.clone());
                assert!(a.is_none());

                // 2. Second insert: Key exists, so it returns Some(data_v1)
                let b = reg.insert(id1, data_v2);
                assert_eq!(b, Some(data_v1)); 
                
                // 3. Verify the registry now holds data_v2
                let current = reg.get(&id1, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = current {
                    assert_eq!(*state, 20.0);
                }
            }

            #[test]
            fn test_bulk_mutation() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(8);
                let id2 = MockId(99);

                reg.insert(id1, MockData::Kind1 { state: 10.0 });
                reg.insert(id2, MockData::Kind1 { state: 20.0 });

                // Multiply all Kind1 states by 2
                // IMPORTANT: must use values_mut_by_kind for mutation
                for data in reg.values_mut_by_kind(MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        *state *= 2.0;
                    }
                }

                // Verify both updated using Option pattern
                if let Some(MockData::Kind1 { state }) = reg.get(&id1, MockKind::Kind1) {
                    assert_eq!(*state, 20.0);
                }
                if let Some(MockData::Kind1 { state }) = reg.get(&id2, MockKind::Kind1) {
                    assert_eq!(*state, 40.0);
                }
            }

            #[test]
            fn test_mutable_access() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(1);

                reg.insert(id1, MockData::Kind1 { state: 10.0 });

                // Get mutably and modify using if-let Some
                if let Some(data) = reg.get_mut(&id1, MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        *state += 5.0;
                    }
                }

                // Verify the change stuck
                if let Some(MockData::Kind1 { state }) = reg.get(&id1, MockKind::Kind1) {
                    assert_eq!(*state, 15.0);
                }
            }

            #[test]
            fn test_iteration_and_filtering() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(1);
                let id2 = MockId(2);
                let id3 = MockId(3);

                reg.insert(id1, MockData::Kind1 { state: 4.0 });
                reg.insert(id2, MockData::Kind1 { state: 2.0 });
                reg.insert(id3, MockData::Kind2 { other: 99 });

                // Test Kind1 collection - using trait methods
                let kind1_count = reg.values_by_kind(MockKind::Kind1).count();
                assert_eq!(kind1_count, 2);

                // Test total collection
                let total_count = reg.values().count();
                assert_eq!(total_count, 3);
            }

            #[test]
            fn test_kinds() {
                let mut reg = TestRegistry::default();
                let id = MockId(1);
                let data1 = MockData::Kind1 { state: 10.0 };
                let data2 = MockData::Kind2 { other: 99 };

                // 1. Insert Kind1 with ID 1 - returns None because it's new
                assert!(reg.insert(id, data1.clone()).is_none());

                // 2. Insert Kind2 with the SAME ID 1 - returns None because (ID, Kind2) is new
                assert!(reg.insert(id, data2).is_none(), "Should allow same ID for different Kinds");

                // 3. Verify both exist independently
                let res1 = reg.get(&id, MockKind::Kind1).unwrap();
                let res2 = reg.get(&id, MockKind::Kind2).unwrap();

                if let MockData::Kind1 { state } = res1 { assert_eq!(*state, 10.0); }
                if let MockData::Kind2 { other } = res2 { assert_eq!(*other, 99); }

                // 4. Verify collision (overwrite) returns the old data
                let duplicate = MockData::Kind1 { state: 20.0 };
                let old_data = reg.insert(id, duplicate);
                assert_eq!(old_data, Some(data1));
            }
        }
    };
}

#[macro_export]
macro_rules! test_model {
    ($registry_type:ident) => {
        #[cfg(test)]
        mod model_tests {
            

            $crate::__define_test_mocks!(); 

             // 3. Define the Test Config
            pub struct TestConfig;
            impl ModelConfig for TestConfig {
                type Id = MockId;
                type Data = MockData;
                type Registry = $registry_type<MockId, MockData>;
                type Category = MockUnitCategory;
                type Setting = MockUnitSetting;
                
              
                type Lang = MockLang;
                type Display = MockDisplayText;
                //type Translator = MockTranslator;
            }
 

            // Helper to initialize the model
            fn create_model() -> Model<TestConfig> {
                let registry = <$registry_type<MockId, MockData>>::default();
                let settings = UnitSettings::new(MockUnitSetting, MockUnitSetting); 
                Model::new(registry, settings)
            }

           

            #[test]
            fn test_flow() {
                let mut model = create_model();

                let id = MockId(1);
                let data = MockData::Kind1 { state: 10.0 };

                // --- TEST INSERT ---
                assert!(model.insert(id, data.clone()).is_ok());
                
                let err = model.insert(id, data.clone()).unwrap_err();
                assert!(matches!(err, ModelError::AlreadyExists(..)));

                // --- TEST GET & UPDATE ---
                let fetched = model.get(&id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = fetched {
                    assert_eq!(*state, 10.0);
                } else {
                    panic!("Expected Kind1 variant");
                }

                let new_data = MockData::Kind1 { state: 1.0 };
                assert!(model.update(&id, new_data).is_ok());
                
                let updated = model.get(&id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = updated {
                    assert_eq!(*state, 1.0);
                }

                // --- TEST DELETE ---
                let removed = model.delete(&id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = removed {
                    assert_eq!(state, 1.0);
                }
                
                assert!(model.get(&id, MockKind::Kind1).is_err());
            }

            #[test]
            fn test_error_on_missing() {
                let mut model = create_model();
                let id = MockId(99);
                let res = model.update(&id, MockData::Kind1 { state: 0.0 });
                assert!(matches!(res, Err(ModelError::NotFound(..))));
            }

            #[test]
            fn test_wrong_kind_lookup() {
                let mut model = create_model();
                let id = MockId(8);
                model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

                let res = model.get(&id, MockKind::Kind2);
                assert!(matches!(res, Err(ModelError::NotFound(..))));
            }

            #[test]
            fn test_clone_independence() {
                let mut model = create_model();
                let id = MockId(8);
                model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

                let mut cloned_data = model.get_clone(&id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = &mut cloned_data {
                    *state = 99.0;
                }

                if let Ok(MockData::Kind1 { state }) = model.get(&id, MockKind::Kind1) {
                    assert_eq!(*state, 10.0);
                }
            }
 
            #[test]
            fn test_model_rejects_zero_id() {
                let mut model = create_model();
                let id = MockId::invalid(); 
                let data = MockData::Kind1 { state: 10.0 };
                
                let result = model.insert(id, data);
                assert!(matches!(result, Err(ModelError::InvalidId(..))));
            }
        }
    };
}


// #[macro_export]
// macro_rules! test_model {
//     ($registry_type:ident) => {
//         #[cfg(test)]
//         mod model_tests {
            
//             $crate::__define_test_mocks!(); 
//             type TestRegistry = $registry_type<MockId, MockData>;

            
//             #[test]
//             fn test_flow() {
                
//                 let mut model = Model::new(TestRegistry::default());

//                 let id = MockId(1);
//                 let data = MockData::Kind1 { state: 10.0 };

//                 // --- TEST INSERT ---
//                 assert!(model.insert(id, data.clone()).is_ok());
                
//                 // Ensure collision detection works
//                 let err = model.insert(id, data.clone()).unwrap_err();
//                 assert!(matches!(err, ModelError::AlreadyExists(..)));

//                 // --- TEST GET & UPDATE ---
//                 // Match on the result to verify the inner state
//                 let fetched = model.get(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = fetched {
//                     assert_eq!(*state, 10.0);
//                 } else {
//                     panic!("Expected Kind1 variant");
//                 }

//                 // Update in place
//                 let new_data = MockData::Kind1 { state: 1.0 };
//                 assert!(model.update(&id, new_data).is_ok());
                
//                 let updated = model.get(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = updated {
//                     assert_eq!(*state, 1.0);
//                 }

//                 // --- TEST DELETE ---
//                 let removed = model.delete(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = removed {
//                     assert_eq!(state, 1.0);
//                 }
                
//                 // Verify it's truly gone
//                 assert!(model.get(&id, MockKind::Kind1).is_err());
//             }

//             #[test]
//             fn test_error_on_missing() {
//                 let mut model = Model::new(TestRegistry::default());
         
//                 let id = MockId(99);
//                 let res = model.update(&id, MockData::Kind1 { state: 0.0 });
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind1))));

//                 let res = model.delete(&id, MockKind::Kind1);
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind1))));
//             }

//             #[test]
//             fn test_wrong_kind_lookup() {
//                 let mut model = Model::new(TestRegistry::default());
//                 let id = MockId(8);
//                 model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

//                 // ID 1 exists, but it is NOT Kind2.
//                 let res = model.get(&id, MockKind::Kind2);
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind2))));
//             }

//             #[test]
//             fn test_clone_independence() {
//                 let mut model = Model::new(TestRegistry::default());

//                 let id = MockId(8);
//                 model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

//                 let mut cloned_data = model.get_clone(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = &mut cloned_data {
//                     *state = 99.0;
//                 }

//                 // Original should still be 10.0
//                 if let Ok(MockData::Kind1 { state }) = model.get(&id, MockKind::Kind1) {
//                     assert_eq!(*state, 10.0);
//                 }
//             }
 
//             #[test]
//             fn test_model_rejects_zero_id() {
//                 let mut model = Model::new(TestRegistry::default());
//                 let id = MockId::invalid(); 
//                 let data =MockData::Kind1 { state: 10.0 };
                
//                 let result = model.insert(id, data);
//                 assert!(matches!(result, Err(ModelError::InvalidId(_, _))));
//             }

             
//         }
//     };
// }
 
 


// #[macro_export]
// macro_rules! test_component_registry {
//     ($registry_type:ident) => {
//         #[cfg(test)]
//         mod registry_tests {
//             use super::*;
           
//             use $crate::model::registry::ComponentRegistry;
//             use $crate::model::component::{ComponentId, ComponentData, ComponentKind};
//             use $crate::model::{Model, ModelError};
//             use std::collections::HashMap;

             
           

//             type TestRegistry = $registry_type<MockId, MockData>;

//             fn get_model() -> Model<MockId, MockData, TestRegistry> {
//                 let registry = HashMapRegistry::new();
//                 Model::new(registry)
//             }

//             #[test]
//             fn test_flow() {
                
//                 let mut model = get_model();

//                 let id = MockId(1);
//                 let data = MockData::Kind1 { state: 10.0 };

//                 // --- TEST INSERT ---
//                 assert!(model.insert(id, data.clone()).is_ok());
                
//                 // Ensure collision detection works
//                 let err = model.insert(id, data.clone()).unwrap_err();
//                 assert!(matches!(err, ModelError::AlreadyExists(..)));

//                 // --- TEST GET & UPDATE ---
//                 // Match on the result to verify the inner state
//                 let fetched = model.get(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = fetched {
//                     assert_eq!(*state, 10.0);
//                 } else {
//                     panic!("Expected Kind1 variant");
//                 }

//                 // Update in place
//                 let new_data = MockData::Kind1 { state: 1.0 };
//                 assert!(model.update(&id, new_data).is_ok());
                
//                 let updated = model.get(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = updated {
//                     assert_eq!(*state, 1.0);
//                 }

//                 // --- TEST DELETE ---
//                 let removed = model.delete(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = removed {
//                     assert_eq!(state, 1.0);
//                 }
                
//                 // Verify it's truly gone
//                 assert!(model.get(&id, MockKind::Kind1).is_err());
//             }

//             #[test]
//             fn test_kinds() {
//                 let mut model = get_model();

//                 let id = MockId(1);
//                 let data1 = MockData::Kind1 { state: 10.0 };
//                 let data2 = MockData::Kind2 { other: 99 };

//                 // 1. Insert Kind1 with ID 1
//                 assert!(model.insert(id, data1).is_ok());

//                 // 2. Insert Kind2 with the SAME ID 1
//                 // This should SUCCEED because they have different Kinds (ComponentKey is different)
//                 assert!(model.insert(id, data2).is_ok(), "Should allow same ID for different Kinds");

//                 // 3. Verify both exist independently
//                 let res1 = model.get(&id, MockKind::Kind1).unwrap();
//                 let res2 = model.get(&id, MockKind::Kind2).unwrap();

//                 if let MockData::Kind1 { state } = res1 { assert_eq!(*state, 10.0); }
//                 if let MockData::Kind2 { other } = res2 { assert_eq!(*other, 99); }

//                 // 4. Verify collision still happens if BOTH ID and Kind match
//                 let duplicate = MockData::Kind1 { state: 20.0 };
//                 let err = model.insert(id, duplicate).unwrap_err();
//                 assert!(matches!(err, ModelError::AlreadyExists(..)));
//             }

//             #[test]
//             fn test_iteration_and_filtering() {
//                 let mut model = get_model();

//                 let id1 = MockId(1);
//                 let id2 = MockId(2);
//                 let id3 = MockId(3);

//                 model.insert(id1, MockData::Kind1 { state: 4.0 }).unwrap();
//                 model.insert(id2, MockData::Kind1 { state: 2.0 }).unwrap();
//                 model.insert(id3, MockData::Kind2 { other: 99 }).unwrap();

//                 // Test Kind1 collection
//                 let kind1_count = model.components_by_kind(MockKind::Kind1).count();
//                 assert_eq!(kind1_count, 2);

//                 // Test total collection
//                 let total_count = model.components().count();
//                 assert_eq!(total_count, 3);
//             }

//             #[test]
//             fn test_mutable_access() {
//                 let mut model = get_model();

//                 let id1 = MockId(1);

//                 model.insert(id1, MockData::Kind1 { state: 10.0 }).unwrap();

//                 // Get mutably and modify
//                 if let Ok(data) = model.get_mut(&id1, MockKind::Kind1) {
//                     if let MockData::Kind1 { state } = data {
//                         *state += 5.0;
//                     }
//                 }

//                 // Verify the change stuck
//                 if let Ok(MockData::Kind1 { state }) = model.get(&id1, MockKind::Kind1) {
//                     assert_eq!(*state, 15.0);
//                 }
//             }

//             #[test]
//             fn test_error_on_missing() {
//                 let mut model = get_model();
         
//                 let id = MockId(99);
//                 let res = model.update(&id, MockData::Kind1 { state: 0.0 });
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind1))));

//                 let res = model.delete(&id, MockKind::Kind1);
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind1))));
//             }

//             #[test]
//             fn test_wrong_kind_lookup() {
//                 let mut model = get_model();
//                 let id = MockId(8);
//                 model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

//                 // ID 1 exists, but it is NOT Kind2.
//                 let res = model.get(&id, MockKind::Kind2);
//                 assert!(matches!(res, Err(ModelError::NotFound(id, MockKind::Kind2))));
//             }

//             #[test]
//             fn test_bulk_mutation() {
//                 let mut model = get_model();
//                 let id1 = MockId(8);
//                 let id2 = MockId(99);

//                 model.insert(id1, MockData::Kind1 { state: 10.0 }).unwrap();
//                 model.insert(id2, MockData::Kind1 { state: 20.0 }).unwrap();

//                 // Multiply all Kind1 states by 2
//                 for data in model.components_mut_by_kind(MockKind::Kind1) {
//                     if let MockData::Kind1 { state } = data {
//                         *state *= 2.0;
//                     }
//                 }

//                 // Verify both updated
//                 if let Ok(MockData::Kind1 { state }) = model.get(&id1, MockKind::Kind1) {
//                     assert_eq!(*state, 20.0);
//                 }
//                 if let Ok(MockData::Kind1 { state }) = model.get(&id2, MockKind::Kind1) {
//                     assert_eq!(*state, 40.0);
//                 }
//             }

//             #[test]
//             fn test_clone_independence() {
//                 let mut model = get_model();

//                 let id = MockId(8);
//                 model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

//                 let mut cloned_data = model.get_clone(&id, MockKind::Kind1).unwrap();
//                 if let MockData::Kind1 { state } = &mut cloned_data {
//                     *state = 99.0;
//                 }

//                 // Original should still be 10.0
//                 if let Ok(MockData::Kind1 { state }) = model.get(&id, MockKind::Kind1) {
//                     assert_eq!(*state, 10.0);
//                 }
//             }
 
//             #[test]
//             fn test_model_rejects_zero_id() {
//                 let mut model = get_model();
//                 let id = MockId::invalid(); 
//                 let data =MockData::Kind1 { state: 10.0 };
                
//                 let result = model.insert(id, data);
//                 assert!(matches!(result, Err(ModelError::InvalidId(_, _))));
//             }

//         }
//     };
// }
 