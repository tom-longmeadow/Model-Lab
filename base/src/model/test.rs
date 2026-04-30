#[macro_export]
#[doc(hidden)]
macro_rules! __define_test_mocks {
    () => { 
        use $crate::model::registry::ComponentRegistry;
        use $crate::model::component::{ComponentId, ComponentKind, HasKind};
        use $crate::model::{Model, ModelError, ModelConfig};

        #[derive(Debug, Default, Clone, Copy)]
        pub enum MockLang { #[default] En }
        impl $crate::language::Language for MockLang {
            fn locale_code(&self) -> &'static str { "en-US" }
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub enum MockDisplayText { #[default] Generic }
        impl $crate::language::DisplayLanguage for MockDisplayText {
            fn default_text(&self) -> &'static str { "Generic" }
            fn translate<L: $crate::language::Language>(&self, _lang: L) -> String {
                self.default_text().to_string()
            }
            fn id_label() -> Self { MockDisplayText::Generic }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub enum MockUnitCategory { #[default] Generic }
        impl $crate::unit::category::UnitCategory for MockUnitCategory {}

        #[derive(Debug, Clone, Copy, PartialEq, Default)]
        pub struct MockUnitSetting;
        impl $crate::unit::settings::UnitSettings<MockUnitCategory> for MockUnitSetting {
            fn default() -> Self { Self }
            fn get(&self, _: MockUnitCategory) -> $crate::unit::kind::UnitKind {
                $crate::unit::kind::UnitKind::Simple($crate::unit::simple::SimpleUnit::length_si())
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct MockId(pub u64);
        impl ComponentId for MockId {
            fn invalid() -> Self { Self(0) }
        }
        impl std::fmt::Display for MockId {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::str::FromStr for MockId {
            type Err = std::num::ParseIntError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(MockId(s.parse()?))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum MockKind { Kind1, Kind2 }
        impl ComponentKind for MockKind {
            type Id = MockId;
        }

        impl std::fmt::Display for MockKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Kind1 => write!(f, "Kind1"),
                Self::Kind2 => write!(f, "Kind2"),
            }
        }
    }

        #[derive(Debug, Clone, PartialEq)]
        pub enum MockData {
            Kind1 { state: f32 }, 
            Kind2 { other: u32 }, 
        }
        impl HasKind for MockData {
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
            use super::*;
            $crate::__define_test_mocks!(); 

            pub struct MockConfig;

            impl $crate::model::config::ModelConfig for MockConfig {
                type Kind = MockKind;
                type Data = MockData;
                type Registry = $registry_type<MockConfig>;
            }

            impl $crate::property::config::PropertyConfig for MockConfig {
                type Display = MockDisplayText;
                type Lang = MockLang;
            }

            impl $crate::unit::config::UnitConfig for MockConfig {
                type UnitCategory = MockUnitCategory;
                type UnitSetting = MockUnitSetting;
            }

            type TestRegistry = $registry_type<MockConfig>;

            #[test]
            fn test_raw_storage() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(8);
                let data_v1 = MockData::Kind1 { state: 10.0_f32 };
                let data_v2 = MockData::Kind1 { state: 20.0_f32 };

                let a = reg.insert(id1, data_v1.clone());
                assert!(a.is_none());

                let b = reg.insert(id1, data_v2);
                assert_eq!(b, Some(data_v1)); 
                
                let current = reg.get(&id1, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = current {
                    assert_eq!(*state, 20.0_f32);
                }
            }

            #[test]
            fn test_bulk_mutation() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(8);
                let id2 = MockId(99);

                reg.insert(id1, MockData::Kind1 { state: 10.0 });
                reg.insert(id2, MockData::Kind1 { state: 20.0 });

                for data in reg.values_mut_by_kind(MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        *state *= 2.0;
                    }
                }

                if let Some(MockData::Kind1 { state }) = reg.get(&id1, MockKind::Kind1) {
                    assert_eq!(*state, 20.0_f32);
                }
                if let Some(MockData::Kind1 { state }) = reg.get(&id2, MockKind::Kind1) {
                    assert_eq!(*state, 40.0_f32);
                }
            }

            #[test]
            fn test_mutable_access() {
                let mut reg = TestRegistry::default();
                let id1 = MockId(1);

                reg.insert(id1, MockData::Kind1 { state: 10.0 });

                if let Some(data) = reg.get_mut(&id1, MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        *state += 5.0_f32;
                    }
                }

                if let Some(MockData::Kind1 { state }) = reg.get(&id1, MockKind::Kind1) {
                    assert_eq!(*state, 15.0_f32);
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

                let kind1_count = reg.values_by_kind(MockKind::Kind1).count();
                assert_eq!(kind1_count, 2);

                let total_count = reg.values().count();
                assert_eq!(total_count, 3);
            }

            #[test]
            fn test_kinds() {
                let mut reg = TestRegistry::default();
                let id = MockId(1);
                let data1 = MockData::Kind1 { state: 10.0 };
                let data2 = MockData::Kind2 { other: 99 };

                assert!(reg.insert(id, data1.clone()).is_none());
                assert!(reg.insert(id, data2).is_none(), "Should allow same ID for different Kinds");

                let res1 = reg.get(&id, MockKind::Kind1).unwrap();
                let res2 = reg.get(&id, MockKind::Kind2).unwrap();

                if let MockData::Kind1 { state } = res1 { assert_eq!(*state, 10.0_f32); }
                if let MockData::Kind2 { other } = res2 { assert_eq!(*other, 99_u32); }

                let duplicate = MockData::Kind1 { state: 20.0 };
                let old_data = reg.insert(id, duplicate);
                assert_eq!(old_data, Some(data1));
            }

            #[test]
            fn test_remove() {
                let mut reg = TestRegistry::default();
                let id = MockId(5);

                reg.insert(id, MockData::Kind1 { state: 42.0 });
                assert!(reg.contains(&id, MockKind::Kind1));

                let removed = reg.remove(&id, MockKind::Kind1);
                assert!(matches!(removed, Some(MockData::Kind1 { state }) if state == 42.0));
                assert!(!reg.contains(&id, MockKind::Kind1));
            }

            #[test]
            fn test_remove_nonexistent_returns_none() {
                let mut reg = TestRegistry::default();
                let id = MockId(99);

                let result = reg.remove(&id, MockKind::Kind1);
                assert!(result.is_none());
            }

            #[test]
            fn test_get_nonexistent_returns_none() {
                let reg = TestRegistry::default();
                let id = MockId(42);

                assert!(reg.get(&id, MockKind::Kind1).is_none());
                assert!(reg.get(&id, MockKind::Kind2).is_none());
            }

            #[test]
            fn test_contains() {
                let mut reg = TestRegistry::default();
                let id = MockId(7);

                assert!(!reg.contains(&id, MockKind::Kind1));
                reg.insert(id, MockData::Kind1 { state: 1.0 });
                assert!(reg.contains(&id, MockKind::Kind1));
                assert!(!reg.contains(&id, MockKind::Kind2));
            }

            #[test]
            fn test_values_by_kind_empty() {
                let reg = TestRegistry::default();
                let count = reg.values_by_kind(MockKind::Kind1).count();
                assert_eq!(count, 0);
            }

            #[test]
            fn test_values_empty() {
                let reg = TestRegistry::default();
                let count = reg.values().count();
                assert_eq!(count, 0);
            }

            #[test]
            fn test_overwrite_does_not_increase_count() {
                let mut reg = TestRegistry::default();
                let id = MockId(1);

                reg.insert(id, MockData::Kind1 { state: 1.0 });
                reg.insert(id, MockData::Kind1 { state: 2.0 });

                let count = reg.values_by_kind(MockKind::Kind1).count();
                assert_eq!(count, 1);
            }

            #[test]
            fn test_remove_one_kind_leaves_other_intact() {
                let mut reg = TestRegistry::default();
                let id = MockId(3);

                reg.insert(id, MockData::Kind1 { state: 1.0 });
                reg.insert(id, MockData::Kind2 { other: 2 });
                reg.remove(&id, MockKind::Kind1);

                assert!(!reg.contains(&id, MockKind::Kind1));
                assert!(reg.contains(&id, MockKind::Kind2));
                assert_eq!(reg.values().count(), 1);
            }

            #[test]
            fn test_values_mut_by_kind_only_affects_target_kind() {
                let mut reg = TestRegistry::default();
                reg.insert(MockId(1), MockData::Kind1 { state: 10.0 });
                reg.insert(MockId(2), MockData::Kind2 { other: 5 });

                for data in reg.values_mut_by_kind(MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        *state = 99.0_f32;
                    }
                }

                // Kind2 should be untouched
                if let Some(MockData::Kind2 { other }) = reg.get(&MockId(2), MockKind::Kind2) {
                    assert_eq!(*other, 5_u32);
                }
            }

            #[test]
            fn test_large_number_of_entries() {
                let mut reg = TestRegistry::default();

                for i in 1u64..=100 {
                    reg.insert(MockId(i), MockData::Kind1 { state: i as f32 });
                }

                assert_eq!(reg.values_by_kind(MockKind::Kind1).count(), 100);
                assert_eq!(reg.values().count(), 100);

                for data in reg.values_by_kind(MockKind::Kind1) {
                    if let MockData::Kind1 { state } = data {
                        assert!(*state >= 1.0_f32 && *state <= 100.0_f32);
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! test_model {
    ($registry_type:ident) => {
        #[cfg(test)]
        mod model_tests {
            use super::*;
            $crate::__define_test_mocks!();

            #[derive(Debug)]
            pub struct MockConfig;

            impl $crate::model::config::ModelConfig for MockConfig {
                type Kind = MockKind;
                type Data = MockData;
                type Registry = $registry_type<MockConfig>;
            }

            impl $crate::property::config::PropertyConfig for MockConfig {
                type Display = MockDisplayText;
                type Lang = MockLang;
            }

            impl $crate::unit::config::UnitConfig for MockConfig {
                type UnitCategory = MockUnitCategory;
                type UnitSetting = MockUnitSetting;
            }

            fn create_model() -> $crate::model::Model<MockConfig> {
                $crate::model::Model::new(
                    $registry_type::new(),
                    $crate::unit::UnitSystem::new(MockUnitSetting::default()),
                )
            }

            #[test]
            fn test_flow() {
                let mut model = create_model();
                let id = MockId(1);
                let data = MockData::Kind1 { state: 10.0 };

                assert!(model.insert(id, data.clone()).is_ok());
                
                let err = model.insert(id, data.clone()).unwrap_err();
                assert!(matches!(err, ModelError::AlreadyExists(..)));

                let fetched = model.get(id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = fetched {
                    assert_eq!(*state, 10.0);
                } else {
                    panic!("Expected Kind1 variant");
                }

                let new_data = MockData::Kind1 { state: 1.0 };
                assert!(model.update(id, new_data).is_ok());
                
                let updated = model.get(id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = updated {
                    assert_eq!(*state, 1.0);
                }

                let removed = model.delete(id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = removed {
                    assert_eq!(state, 1.0);
                }
                
                assert!(model.get(id, MockKind::Kind1).is_err());
            }

            #[test]
            fn test_error_on_missing() {
                let mut model = create_model();
                let id = MockId(99);
                let res = model.update(id, MockData::Kind1 { state: 0.0 });
                assert!(matches!(res, Err(ModelError::NotFound(..))));
            }

            #[test]
            fn test_wrong_kind_lookup() {
                let mut model = create_model();
                let id = MockId(8);
                model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

                let res = model.get(id, MockKind::Kind2);
                assert!(matches!(res, Err(ModelError::NotFound(..))));
            }

            #[test]
            fn test_clone_independence() {
                let mut model = create_model();
                let id = MockId(8);
                model.insert(id, MockData::Kind1 { state: 10.0 }).unwrap();

                let mut cloned_data = model.get_clone(id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = &mut cloned_data {
                    *state = 99.0;
                }

                if let Ok(MockData::Kind1 { state }) = model.get(id, MockKind::Kind1) {
                    assert_eq!(*state, 10.0);
                }
            }

            #[test]
            fn test_model_rejects_invalid_id() {
                let mut model = create_model();
                let id = MockId::invalid(); 
                let data = MockData::Kind1 { state: 10.0 };
                
                let result = model.insert(id, data);
                assert!(matches!(result, Err(ModelError::InvalidId(..))));
            }

            #[test]
            fn test_same_id_different_kinds_allowed() {
                let mut model = create_model();
                let id = MockId(1);

                assert!(model.insert(id, MockData::Kind1 { state: 1.0 }).is_ok());
                assert!(model.insert(id, MockData::Kind2 { other: 2 }).is_ok());

                assert!(model.get(id, MockKind::Kind1).is_ok());
                assert!(model.get(id, MockKind::Kind2).is_ok());
            }

            #[test]
            fn test_delete_nonexistent_returns_error() {
                let mut model = create_model();
                let id = MockId(55);

                let res = model.delete(id, MockKind::Kind1);
                assert!(matches!(res, Err(ModelError::NotFound(..))));
            }

            #[test]
            fn test_delete_removes_only_target_kind() {
                let mut model = create_model();
                let id = MockId(3);

                model.insert(id, MockData::Kind1 { state: 1.0 }).unwrap();
                model.insert(id, MockData::Kind2 { other: 2 }).unwrap();

                model.delete(id, MockKind::Kind1).unwrap();

                assert!(model.get(id, MockKind::Kind1).is_err());
                assert!(model.get(id, MockKind::Kind2).is_ok());
            }

            #[test]
            fn test_update_invalid_id_rejected() {
                let mut model = create_model();
                let id = MockId::invalid();

                let res = model.update(id, MockData::Kind1 { state: 1.0 });
                assert!(matches!(res, Err(ModelError::InvalidId(..))));
            }

            #[test]
            fn test_update_preserves_kind() {
                let mut model = create_model();
                let id = MockId(10);

                model.insert(id, MockData::Kind1 { state: 5.0 }).unwrap();
                model.update(id, MockData::Kind1 { state: 50.0 }).unwrap();

                let fetched = model.get(id, MockKind::Kind1).unwrap();
                if let MockData::Kind1 { state } = fetched {
                    assert_eq!(*state, 50.0);
                }
            }

            #[test]
            fn test_get_clone_returns_independent_copy() {
                let mut model = create_model();
                let id = MockId(20);

                model.insert(id, MockData::Kind2 { other: 42 }).unwrap();

                let clone = model.get_clone(id, MockKind::Kind2).unwrap();
                assert!(matches!(clone, MockData::Kind2 { other: 42 }));
            }

            #[test]
            fn test_multiple_inserts_and_deletes() {
                let mut model = create_model();

                for i in 1u64..=10 {
                    model.insert(MockId(i), MockData::Kind1 { state: i as f32 }).unwrap();
                }

                for i in 1u64..=5 {
                    model.delete(MockId(i), MockKind::Kind1).unwrap();
                }

                for i in 1u64..=5 {
                    assert!(model.get(MockId(i), MockKind::Kind1).is_err());
                }
                for i in 6u64..=10 {
                    assert!(model.get(MockId(i), MockKind::Kind1).is_ok());
                }
            }
        }
    };
}

 