use crate::{
    test_core, TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_MODULE_RESPONSE, TEST_OS_ID,
    TEST_PROXY, TEST_VERSION_CONTROL,
};
use abstract_os::manager::state::OS_ID;
use abstract_os::{manager::state::OS_MODULES, version_control::state::OS_ADDRESSES};
use cosmwasm_std::{
    from_binary, testing::MockQuerier, to_binary, Addr, Binary, ContractResult, Empty,
    QuerierWrapper, SystemResult, WasmQuery,
};
use cw_storage_plus::{Item, Map, PrimaryKey};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;

pub type EmptyMockQuerier = MockQuerier<Empty>;

type BinaryQueryResult = Result<Binary, String>;
type ContractAddr = String;
type FallbackHandler = dyn for<'a> Fn(&'a str, &'a Binary) -> BinaryQueryResult;
type SmartHandler = dyn for<'a> Fn(&'a Binary) -> BinaryQueryResult;
type RawHandler = dyn for<'a> Fn(&'a str) -> BinaryQueryResult;

/// [`MockQuerierBuilder`] is a helper to build a [`MockQuerier`].
pub struct MockQuerierBuilder {
    base: EmptyMockQuerier,
    fallback_raw_handler: Box<FallbackHandler>,
    fallback_smart_handler: Box<FallbackHandler>,
    smart_handlers: HashMap<ContractAddr, Box<SmartHandler>>,
    raw_handlers: HashMap<ContractAddr, Box<RawHandler>>,
    raw_mappings: HashMap<ContractAddr, HashMap<Binary, Binary>>,
}

impl Default for MockQuerierBuilder {
    /// Create a default
    fn default() -> Self {
        let raw_fallback: fn(&str, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(&key.0).unwrap();
            Err(format!(
                "No mock querier for this query: {addr:?} {str_key:?}"
            ))
        };
        let smart_fallback: fn(&str, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(&key.0).unwrap();
            Err(format!("unexpected contract: {addr:?} {str_key:?}"))
        };

        Self {
            base: MockQuerier::default(),
            fallback_raw_handler: Box::from(raw_fallback),
            fallback_smart_handler: Box::from(smart_fallback),
            smart_handlers: HashMap::default(),
            raw_handlers: HashMap::default(),
            raw_mappings: HashMap::default(),
        }
    }
}

pub fn map_key<'a, K, V>(map: &Map<'a, K, V>, key: K) -> String
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    String::from_utf8(raw_map_key(map, key)).unwrap()
}

pub fn raw_map_key<'a, K, V>(map: &Map<'a, K, V>, key: K) -> Vec<u8>
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    map.key(key).deref().to_vec()
}

/// Helper to build a MockQuerier.
/// Usage:
/// ```rust
/// use cosmwasm_std::{from_binary, to_binary};
/// use abstract_testing::MockQuerierBuilder;
/// use cosmwasm_std::testing::MockQuerier;
/// use abstract_testing::mock_module::MockModuleExecuteMsg;
///
/// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
///    // handle the message
///     let res = match from_binary::<MockModuleExecuteMsg>(msg).unwrap() {
///         // handle the message
///        _ => panic!("unexpected message"),
///    };
///
///   Ok(to_binary(&msg).unwrap())
/// }).build();
/// ```
impl MockQuerierBuilder {
    pub fn with_fallback_smart_handler<SH: 'static>(mut self, handler: SH) -> Self
    where
        SH: Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_smart_handler = Box::new(handler);
        self
    }

    pub fn with_fallback_raw_handler<RH: 'static>(mut self, handler: RH) -> Self
    where
        RH: Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_raw_handler = Box::new(handler);
        self
    }

    /// Add a smart contract handler to the mock querier. The handler will be called when the
    /// contract address is queried with the given message.
    /// Usage:
    /// ```rust
    /// use cosmwasm_std::{from_binary, to_binary};
    /// use abstract_testing::MockQuerierBuilder;
    /// use cosmwasm_std::testing::MockQuerier;
    /// use abstract_testing::mock_module::MockModuleExecuteMsg;
    ///
    /// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
    ///    // handle the message
    ///     let res = match from_binary::<MockModuleExecuteMsg>(msg).unwrap() {
    ///         // handle the message
    ///        _ => panic!("unexpected message"),
    ///    };
    ///
    ///   Ok(to_binary(&msg).unwrap())
    /// }).build();
    /// ```
    pub fn with_smart_handler<SH: 'static>(mut self, contract: &str, handler: SH) -> Self
    where
        SH: Fn(&Binary) -> BinaryQueryResult,
    {
        self.smart_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    pub fn with_raw_handler<RH: 'static>(mut self, contract: &str, handler: RH) -> Self
    where
        RH: Fn(&str) -> BinaryQueryResult,
    {
        self.raw_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    fn insert_contract_key_value(&mut self, contract: &str, key: Vec<u8>, value: Binary) {
        let raw_map = self.raw_mappings.entry(contract.to_string()).or_default();
        raw_map.insert(Binary(key), value);
    }

    /// Add a map entry to the querier for the given contract.
    /// ```rust
    /// use cw_storage_plus::Map;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// const MAP: Map<String, String> = Map::new("map");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_map_entry(
    ///     "contract_address",
    ///     MAP,
    ///     ("key".to_string(), &"value".to_string())
    /// );
    pub fn with_contract_map_entry<'a, K, V>(
        self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        entry: (K, &V),
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        self.with_contract_map_entries(contract, cw_map, vec![entry])
    }

    pub fn with_contract_map_entries<'a, K, V>(
        mut self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        entries: Vec<(K, &V)>,
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        for (key, value) in entries {
            self.insert_contract_key_value(
                contract,
                raw_map_key(&cw_map, key),
                to_binary(value).unwrap(),
            );
        }

        self
    }

    /// Add an empty map key to the querier for the given contract.
    /// This is useful when you want the item to exist, but not have a value.
    pub fn with_contract_map_key<'a, K, V>(
        mut self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        key: K,
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, raw_map_key(&cw_map, key), Binary(vec![]));

        self
    }

    /// Add an empty item key to the querier for the given contract.
    /// This is useful when you want the item to exist, but not have a value.
    pub fn with_empty_contract_item<T>(mut self, contract: &str, cw_item: Item<T>) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, cw_item.as_slice().to_vec(), Binary(vec![]));

        self
    }

    /// Include a contract item in the mock querier.
    /// ```rust
    /// use cw_storage_plus::Item;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// const ITEM: Item<String> = Item::new("item");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_item(
    ///     "contract_address",
    ///     ITEM,
    ///     &"value".to_string(),
    /// );
    /// ```
    pub fn with_contract_item<T>(mut self, contract: &str, cw_item: Item<T>, value: &T) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(
            contract,
            cw_item.as_slice().to_vec(),
            to_binary(value).unwrap(),
        );

        self
    }

    /// Build the [`MockQuerier`].
    pub fn build(mut self) -> EmptyMockQuerier {
        self.base.update_wasm(move |wasm| {
            let res = match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let str_key = std::str::from_utf8(&key.0).unwrap();

                    // First check for raw mappings
                    if let Some(raw_map) = self.raw_mappings.get(contract_addr.as_str()) {
                        if let Some(value) = raw_map.get(key) {
                            return SystemResult::Ok(ContractResult::Ok(value.clone()));
                        }
                    }

                    // Then check the handlers
                    let raw_handler = self.raw_handlers.get(contract_addr.as_str());

                    match raw_handler {
                        Some(handler) => (*handler)(str_key),
                        None => (*self.fallback_raw_handler)(contract_addr.as_str(), key),
                    }
                }
                WasmQuery::Smart { contract_addr, msg } => {
                    let contract_handler = self.smart_handlers.get(contract_addr.as_str());

                    let res = match contract_handler {
                        Some(handler) => (*handler)(msg),
                        None => (*self.fallback_smart_handler)(contract_addr.as_str(), msg),
                    };
                    res
                }
                // TODO: contract info
                unexpected => panic!("Unexpected query: {unexpected:?}"),
            };

            match res {
                Ok(res) => SystemResult::Ok(ContractResult::Ok(res)),
                Err(e) => SystemResult::Ok(ContractResult::Err(e)),
            }
        });
        self.base
    }
}

/// A mock querier that returns the following responses for the following **RAW** contract -> queries:
/// - TEST_PROXY
///   - "admin" -> TEST_MANAGER
/// - TEST_MANAGER
///   - "os_modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "os_id" -> TEST_OS_ID
/// - TEST_VERSION_CONTROL
///   - "os_core" -> { TEST_PROXY, TEST_MANAGER }
pub fn mock_querier() -> EmptyMockQuerier {
    let raw_handler = |contract: &str, key: &Binary| {
        let _str_key = std::str::from_utf8(&key.0).unwrap();
        match contract {
            TEST_PROXY => Err("unexpected key".to_string()),
            TEST_MANAGER => {
                // Return the default value
                Ok(Binary(vec![]))
            }
            TEST_VERSION_CONTROL => {
                // Default value
                Ok(Binary(vec![]))
            }
            _ => Err("unexpected contract".to_string()),
        }
    };

    MockQuerierBuilder::default()
        .with_fallback_raw_handler(raw_handler)
        .with_contract_map_entry(
            TEST_VERSION_CONTROL,
            OS_ADDRESSES,
            (TEST_OS_ID, &test_core()),
        )
        .with_contract_item(
            TEST_PROXY,
            Item::new("admin"),
            &Some(Addr::unchecked(TEST_MANAGER)),
        )
        .with_contract_item(TEST_MANAGER, OS_ID, &TEST_OS_ID)
        .with_smart_handler(TEST_MODULE_ADDRESS, |msg| {
            let Empty {} = from_binary(msg).unwrap();
            Ok(to_binary(TEST_MODULE_RESPONSE).unwrap())
        })
        .with_contract_map_entry(
            TEST_MANAGER,
            OS_MODULES,
            (TEST_MODULE_ID, &Addr::unchecked(TEST_MODULE_ADDRESS)),
        )
        .build()
}

pub fn wrap_querier(querier: &EmptyMockQuerier) -> QuerierWrapper<'_, Empty> {
    QuerierWrapper::<Empty>::new(querier)
}

#[cfg(test)]
mod tests {
    use crate::{TEST_MODULE_ID, TEST_OS_ID};

    use super::*;
    use abstract_os::manager::state::OS_MODULES;
    use abstract_os::proxy::state::OS_ID;
    use abstract_os::version_control::state::OS_ADDRESSES;
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    mod os_core {
        use super::*;
        use abstract_os::version_control::Core;

        #[test]
        fn should_return_os_address() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = OS_ADDRESSES.query(
                &wrap_querier(&deps.querier),
                Addr::unchecked(TEST_VERSION_CONTROL),
                TEST_OS_ID,
            );

            let expected = Core {
                proxy: Addr::unchecked(TEST_PROXY),
                manager: Addr::unchecked(TEST_MANAGER),
            };

            assert_that!(actual).is_ok().is_some().is_equal_to(expected)
        }
    }

    mod os_id {
        use super::*;

        #[test]
        fn should_return_test_os_id_with_test_manager() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();
            let actual = OS_ID.query(&wrap_querier(&deps.querier), Addr::unchecked(TEST_MANAGER));

            assert_that!(actual).is_ok().is_equal_to(TEST_OS_ID);
        }
    }

    mod os_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = OS_MODULES.query(
                &wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
                TEST_MODULE_ID,
            );

            assert_that!(actual)
                .is_ok()
                .is_some()
                .is_equal_to(Addr::unchecked(TEST_MODULE_ADDRESS));
        }

        // #[test]
        // fn should_return_none_for_unknown_module() {
        //     let mut deps = mock_dependencies();
        //     deps.querier = querier();
        //
        //     let actual = OS_MODULES.query(
        //         &wrap_querier(&deps.querier),
        //         Addr::unchecked(TEST_MANAGER),
        //         "unknown_module",
        //     );
        //
        //     assert_that!(actual).is_ok().is_none();
        // }
    }
}
