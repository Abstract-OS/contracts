//! # Memory Entry
//! An entry (value) in the memory key-value store.

use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
};

use cosmwasm_std::{Addr, Deps, StdError, StdResult};
use cw_asset::AssetInfo;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::memory::Memory;

/// Information on an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetEntry(String);

impl AssetEntry {
    pub fn new<T: ToString>(entry: T) -> Self {
        Self(entry.to_string().to_ascii_lowercase())
    }
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<AssetInfo> {
        memory.query_asset(deps, &self.0)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for AssetEntry {
    fn from(entry: String) -> Self {
        Self::new(entry)
    }
}

impl ToString for AssetEntry {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Key to get the Address of a contract
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct UncheckedContractEntry {
    pub protocol: String,
    pub contract: String,
}

impl UncheckedContractEntry {
    pub fn new<T: ToString>(protocol: T, contract: T) -> Self {
        Self {
            protocol: protocol.to_string(),
            contract: contract.to_string(),
        }
    }
    pub fn check(self, deps: Deps, memory: &Memory) -> Result<ContractEntry, StdError> {
        let entry = ContractEntry {
            contract: self.contract.to_ascii_lowercase(),
            protocol: self.protocol.to_ascii_lowercase(),
        };
        entry.resolve(deps, memory)?;
        Ok(entry)
    }
}

impl TryFrom<String> for UncheckedContractEntry {
    type Error = StdError;
    fn try_from(entry: String) -> Result<Self, Self::Error> {
        let composite: Vec<&str> = entry.split('/').collect();
        if composite.len() != 2 {
            return Err(StdError::generic_err(
                "contract entry should be formatted as \"protocol/contract_name\".",
            ));
        }
        Ok(Self::new(composite[0], composite[1]))
    }
}

/// Key to get the Address of a contract
/// Use [`UncheckedContractEntry`] to construct this type.  
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct ContractEntry {
    pub protocol: String,
    pub contract: String,
}

impl ContractEntry {
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Addr> {
        memory.query_contract(deps, self)
    }
}

impl Display for ContractEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.protocol, self.contract)
    }
}

impl<'a> PrimaryKey<'a> for ContractEntry {
    type Prefix = String;

    type SubPrefix = ();

    type Suffix = String;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.protocol.key();
        keys.extend(self.contract.key());
        keys
    }
}

impl<'a> Prefixer<'a> for ContractEntry {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.protocol.prefix();
        res.extend(self.contract.prefix().into_iter());
        res
    }
}

impl KeyDeserialize for ContractEntry {
    type Output = Self;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(Self {
            protocol: String::from_vec(tu)?,
            contract: String::from_vec(u)?,
        })
    }
}

#[inline(always)]
fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> ContractEntry {
        ContractEntry {
            protocol: "abstract".to_string(),
            contract: "rocket-ship".to_string(),
        }
    }

    fn mock_keys() -> (ContractEntry, ContractEntry, ContractEntry) {
        (
            ContractEntry {
                protocol: "abstract".to_string(),
                contract: "sailing-ship".to_string(),
            },
            ContractEntry {
                protocol: "abstract".to_string(),
                contract: "rocket-ship".to_string(),
            },
            ContractEntry {
                protocol: "shitcoin".to_string(),
                contract: "pump'n dump".to_string(),
            },
        )
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<ContractEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key.clone(), &42069)
            .unwrap();

        assert_eq!(map.load(deps.as_ref().storage, key.clone()).unwrap(), 42069);

        let items = map
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0], (key, 42069));
    }

    #[test]
    fn composite_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<(ContractEntry, Addr), u64> = Map::new("map");

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("larry")),
            &42069,
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("jake")),
            &69420,
        )
        .unwrap();

        let items = map
            .prefix(key)
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
        assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
    }

    #[test]
    fn partial_key_works() {
        let mut deps = mock_dependencies();
        let (key1, key2, key3) = mock_keys();
        let map: Map<ContractEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        let items = map
            .prefix("abstract".to_string())
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("rocket-ship".to_string(), 69420));
        assert_eq!(items[1], ("sailing-ship".to_string(), 42069));
    }
}
