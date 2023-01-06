use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Storage};

use abstract_os::ans_host::state::{Config, ADMIN, ASSET_PAIRINGS, CONFIG, POOL_METADATA};
use abstract_os::ans_host::{
    AssetPairingFilter, AssetPairingMapEntry, ConfigResponse, PoolAddressListResponse,
    PoolMetadataFilter, PoolMetadataListResponse, PoolMetadataMapEntry, PoolMetadatasResponse,
    PoolsResponse, RegisteredDexesResponse,
};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_metadata::PoolMetadata;
use abstract_os::objects::pool_reference::PoolReference;
use abstract_os::objects::{DexAssetPairing, UniquePoolId};
use abstract_os::{
    ans_host::{
        state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES, REGISTERED_DEXES},
        AssetListResponse, AssetsResponse, ChannelListResponse, ChannelsResponse,
        ContractListResponse, ContractsResponse,
    },
    objects::{AssetEntry, ChannelEntry, ContractEntry},
};
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;

pub(crate) const DEFAULT_LIMIT: u8 = 15;
pub(crate) const MAX_LIMIT: u8 = 25;

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let Config {
        next_unique_pool_id,
    } = CONFIG.load(deps.storage)?;

    let admin = ADMIN.get(deps)?.unwrap();

    let res = ConfigResponse {
        next_unique_pool_id,
        admin,
    };

    to_binary(&res)
}

pub fn query_assets(deps: Deps, _env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets: Vec<AssetEntry> = asset_names
        .iter()
        .map(|name| name.as_str().into())
        .collect();
    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| assets.contains(&e.as_ref().unwrap().0))
        .collect();
    to_binary(&AssetsResponse { assets: res? })
}

pub fn query_contract(deps: Deps, _env: Env, names: Vec<ContractEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ContractsResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel(deps: Deps, _env: Env, names: Vec<ChannelEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ChannelsResponse { channels: res? })
}

pub fn query_asset_list(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_asset_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&AssetListResponse { assets: res? })
}

pub fn query_contract_list(
    deps: Deps,
    last_contract: Option<ContractEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_contract.map(Bound::exclusive);

    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ContractListResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel_list(
    deps: Deps,
    last_channel: Option<ChannelEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_channel.map(Bound::exclusive);

    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ChannelListResponse { channels: res? })
}

pub fn query_registered_dexes(deps: Deps, _env: Env) -> StdResult<Binary> {
    let dexes = REGISTERED_DEXES.load(deps.storage)?;

    to_binary(&RegisteredDexesResponse { dexes })
}

pub fn list_pool_entries(
    deps: Deps,
    filter: Option<AssetPairingFilter>,
    page_token: Option<DexAssetPairing>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let (asset_pair_filter, dex_filter) = match filter {
        Some(AssetPairingFilter { asset_pair, dex }) => (asset_pair, dex),
        None => (None, None),
    };

    let full_key_provided = asset_pair_filter.is_some() && dex_filter.is_some();

    let entry_list: Vec<AssetPairingMapEntry> = if full_key_provided {
        // We have the full key, so load the entry
        let (asset_x, asset_y) = asset_pair_filter.unwrap();
        let key = DexAssetPairing::new(asset_x, asset_y, &dex_filter.unwrap());
        let entry = load_asset_pairing_entry(deps.storage, key)?;
        // Add the result to a vec
        vec![entry]
    } else if let Some((asset_x, asset_y)) = asset_pair_filter {
        let start_bound = page_token.map(|pairing| Bound::exclusive(pairing.dex()));

        // We can use the prefix to load all the entries for the asset pair
        let res: Result<Vec<(DexName, Vec<PoolReference>)>, _> = ASSET_PAIRINGS
            .prefix((asset_x.clone(), asset_y.clone()))
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(page_size)
            .collect();

        // Re add the key prefix, since only the dex is returned as a key
        let matched: Vec<AssetPairingMapEntry> = res?
            .into_iter()
            .map(|(dex, ids)| {
                (
                    DexAssetPairing::new(asset_x.clone(), asset_y.clone(), &dex),
                    ids,
                )
            })
            .collect();

        matched
    } else {
        let start_bound: Option<Bound<DexAssetPairing>> = page_token.map(Bound::exclusive);

        // We have no filter, so load all the entries
        let res: Result<Vec<AssetPairingMapEntry>, _> = ASSET_PAIRINGS
            .range(deps.storage, start_bound, None, Order::Ascending)
            .filter(|e| {
                let pairing = &e.as_ref().unwrap().0;
                dex_filter.as_ref().map_or(true, |f| f == pairing.dex())
            })
            // TODO: is this necessary?
            .map(|e| e.map(|(k, v)| (k, v)))
            .collect();
        res?
    };

    to_binary(&PoolAddressListResponse { pools: entry_list })
}

/// Query the pool ids based on the actual keys
pub fn query_pool_entries(deps: Deps, keys: Vec<DexAssetPairing>) -> StdResult<Binary> {
    let mut entries: Vec<AssetPairingMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_asset_pairing_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolsResponse { pools: entries })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_asset_pairing_entry(
    storage: &dyn Storage,
    key: DexAssetPairing,
) -> StdResult<AssetPairingMapEntry> {
    let value = ASSET_PAIRINGS.load(storage, key.clone())?;
    Ok((key, value))
}

pub fn query_pool_metadatas(deps: Deps, keys: Vec<UniquePoolId>) -> StdResult<Binary> {
    let mut entries: Vec<PoolMetadataMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_pool_metadata_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolMetadatasResponse { metadatas: entries })
}

pub fn list_pool_metadata_entries(
    deps: Deps,
    filter: Option<PoolMetadataFilter>,
    page_token: Option<UniquePoolId>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = page_token.map(Bound::exclusive);

    let pool_type_filter = match filter {
        Some(PoolMetadataFilter { pool_type }) => pool_type,
        None => None,
    };

    let res: Result<Vec<(UniquePoolId, PoolMetadata)>, _> = POOL_METADATA
        // If the asset_pair_filter is provided, we must use that prefix...
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter(|e| {
            let pool_type = &e.as_ref().unwrap().1.pool_type;
            pool_type_filter.as_ref().map_or(true, |f| f == pool_type)
        })
        .take(page_size)
        .map(|e| e.map(|(k, v)| (k, v)))
        .collect();

    to_binary(&PoolMetadataListResponse { metadatas: res? })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_pool_metadata_entry(
    storage: &dyn Storage,
    key: UniquePoolId,
) -> StdResult<PoolMetadataMapEntry> {
    let value = POOL_METADATA.load(storage, key)?;
    Ok((key, value))
}
#[cfg(test)]
mod test {
    use abstract_os::ans_host::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi};
    use cosmwasm_std::{from_binary, DepsMut};

    use crate::contract;
    use crate::contract::{instantiate, AnsHostResult};
    use crate::error::AnsHostError;

    use abstract_os::objects::pool_id::PoolIdBase;
    use cw_asset::{AssetInfoBase, AssetInfoUnchecked};
    use speculoos::prelude::*;

    use super::*;

    type AnsHostTestResult = Result<(), AnsHostError>;

    const TEST_CREATOR: &str = "creator";

    fn mock_init(mut deps: DepsMut) -> AnsHostResult {
        let info = mock_info(TEST_CREATOR, &[]);

        instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        let res = contract::query(deps, mock_env(), msg)?;
        Ok(res)
    }

    fn query_asset_list_msg(token: String, size: usize) -> QueryMsg {
        let msg = QueryMsg::AssetList {
            page_token: (Some(token.to_string())),
            page_size: (Some(size as u8)),
        };
        msg
    }

    fn create_test_assets(
        input: Vec<(&str, &str)>,
        api: MockApi,
    ) -> Vec<(String, AssetInfoBase<Addr>)> {
        let test_assets: Vec<(String, AssetInfoBase<Addr>)> = input
            .into_iter()
            .map(|input| {
                (
                    input.0.to_string().clone().into(),
                    (AssetInfoUnchecked::native(input.1.to_string().clone()))
                        .check(&api, None)
                        .unwrap()
                        .into(),
                )
            })
            .collect();
        test_assets
    }

    fn create_asset_response(test_assets: Vec<(String, AssetInfoBase<Addr>)>) -> AssetsResponse {
        let expected = AssetsResponse {
            assets: test_assets
                .iter()
                .map(|item| (item.0.clone().into(), item.1.clone().into()))
                .collect(),
        };
        expected
    }

    fn create_asset_list_response(
        test_assets: Vec<(String, AssetInfoBase<Addr>)>,
    ) -> AssetListResponse {
        let expected = AssetListResponse {
            assets: test_assets
                .iter()
                .map(|item| (item.0.clone().into(), item.1.clone()))
                .collect(),
        };
        expected
    }
    fn create_contract_entry_and_string(
        input: Vec<(&str, &str, &str)>,
    ) -> Vec<(ContractEntry, String)> {
        let contract_entry: Vec<(ContractEntry, String)> = input
            .into_iter()
            .map(|input| {
                (
                    ContractEntry {
                        protocol: input.0.to_string().to_ascii_lowercase().clone(),
                        contract: input.1.to_string().to_ascii_lowercase().clone(),
                    },
                    input.2.to_string().clone(),
                )
            })
            .collect();
        contract_entry
    }
    fn create_contract_entry(input: Vec<(&str, &str)>) -> Vec<ContractEntry> {
        let contract_entry: Vec<ContractEntry> = input
            .into_iter()
            .map(|input| ContractEntry {
                protocol: input.0.to_string().to_ascii_lowercase().clone(),
                contract: input.1.to_string().to_ascii_lowercase().clone(),
            })
            .collect();
        contract_entry
    }

    fn create_channel_entry_and_string(
        input: Vec<(&str, &str, &str)>,
    ) -> Vec<(ChannelEntry, String)> {
        let channel_entry: Vec<(ChannelEntry, String)> = input
            .into_iter()
            .map(|input| {
                (
                    ChannelEntry {
                        connected_chain: input.0.to_string().to_ascii_lowercase().clone(),
                        protocol: input.1.to_string().to_ascii_lowercase().clone(),
                    },
                    input.2.to_string().clone(),
                )
            })
            .collect();
        channel_entry
    }
    fn create_channel_entry(input: Vec<(&str, &str)>) -> Vec<ChannelEntry> {
        let channel_entry: Vec<ChannelEntry> = input
            .into_iter()
            .map(|input| ChannelEntry {
                connected_chain: input.0.to_string().to_ascii_lowercase().clone(),
                protocol: input.1.to_string().to_ascii_lowercase().clone(),
            })
            .collect();
        channel_entry
    }
    fn create_channel_msg(input: Vec<(&str, &str)>) -> QueryMsg {
        let msg = QueryMsg::Channels {
            names: create_channel_entry(input),
        };
        msg
    }

    fn create_option_pool_ref(id: u64, pool_id: &str, api: MockApi) -> Option<Vec<PoolReference>> {
        let pool_ref = Some(vec![PoolReference {
            id: UniquePoolId::new(id),
            pool_id: PoolIdBase::contract(pool_id).check(&api).unwrap(),
        }]);
        pool_ref
    }
    fn create_pool_metadata(_dex: &str, asset_x: &str, asset_y: &str) -> PoolMetadata {
        let pool_metadata = PoolMetadata::new(
            "bar",
            abstract_os::objects::PoolType::Stable,
            &vec![asset_x.to_string(), asset_y.to_string()],
        );
        pool_metadata
    }
    #[test]
    fn test_query_assets() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(vec![("bar", "bar"), ("foo", "foo")], api);
        for (test_asset_name, test_asset_info) in test_assets.clone().into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
        }
        // create msg
        let msg = QueryMsg::Assets {
            names: vec!["bar".to_string(), "foo".to_string()],
        };
        // send query message
        let res: AssetsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = create_asset_response(test_assets);
        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[test]
    fn test_query_contract() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_contract_entry_and_string(vec![("foo", "foo", "foo")]);
        for (key, new_address) in to_add.into_iter() {
            let addr = deps.as_ref().api.addr_validate(&new_address)?;
            let insert = |_| -> StdResult<Addr> { Ok(addr) };
            CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
        }

        // create, send and deserialise msg
        let msg = QueryMsg::Contracts {
            names: create_contract_entry(vec![("foo", "foo")]),
        };
        let res: ContractsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ContractsResponse {
            contracts: create_contract_entry_and_string(vec![("foo", "foo", "foo")]),
        };

        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[test]
    fn test_query_channels() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        for (key, new_channel) in to_add.into_iter() {
            // Update function for new or existing keys
            let insert = |_| -> StdResult<String> { Ok(new_channel) };
            CHANNELS.update(&mut deps.storage, key, insert)?;
        }
        // create duplicate entry
        let to_add1 = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        for (key, new_channel) in to_add1.into_iter() {
            // Update function for new or existing keys
            let insert = |_| -> StdResult<String> { Ok(new_channel) };
            CHANNELS.update(&mut deps.storage, key, insert)?;
        }

        // create and send and deserialise msg
        let msg = create_channel_msg(vec![("foo", "foo")]);
        let res: ChannelsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ChannelsResponse {
            channels: create_channel_entry_and_string(vec![("foo", "foo", "foo")]),
        };
        // Assert
        assert_that!(&res).is_equal_to(&expected);
        // Assert no duplication
        assert!(res.channels.len() == 1 as usize);
        Ok(())
    }

    #[test]
    fn test_query_asset_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(vec![("foo", "foo"), ("bar", "bar")], api);
        for (test_asset_name, test_asset_info) in test_assets.clone().into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
        }
        // create second entry
        let test_assets1 = create_test_assets(vec![("foobar", "foobar")], api);
        for (test_asset_name, test_asset_info) in test_assets1.clone().into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
        }
        // create duplicate entry
        let test_assets_duplicate = create_test_assets(vec![("foobar", "foobar")], api);
        for (test_asset_name, test_asset_info) in test_assets_duplicate.clone().into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
        }

        // create msgs

        // return all entries
        let msg = query_asset_list_msg("".to_string(), 42);
        let res: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;
        // limit response to 1st result - entries are stored alphabetically
        let msg = query_asset_list_msg("".to_string(), 1);
        let res_first_entry: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // results after specified entry
        let msg = query_asset_list_msg("foo".to_string(), 1);
        let res_of_foobar: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = create_asset_list_response(create_test_assets(
            vec![("bar", "bar"), ("foo", "foo"), ("foobar", "foobar")],
            api,
        ));

        let expected_foobar =
            create_asset_list_response(create_test_assets(vec![("foobar", "foobar")], api));
        let expected_bar =
            create_asset_list_response(create_test_assets(vec![("bar", "bar")], api));

        assert_that!(res).is_equal_to(&expected);
        assert_that!(res_first_entry).is_equal_to(&expected_bar);
        assert_that!(&res_of_foobar).is_equal_to(&expected_foobar);

        Ok(())
    }
    #[test]
    fn test_query_asset_list_above_max() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let generate_test_assets_large = |n: usize| -> Vec<(String, String)> {
            let mut vector = vec![];
            for i in 0..n {
                let string1 = format!("foo{}", i);
                let string2 = format!("foo{}", i);
                vector.push((string1, string2));
            }
            vector
        };
        let test_assets_large: Vec<(String, AssetInfoBase<Addr>)> = generate_test_assets_large(30)
            .into_iter()
            .map(|input| {
                (
                    input.0.clone().into(),
                    (AssetInfoUnchecked::native(input.1.clone()))
                        .check(&api, None)
                        .unwrap()
                        .into(),
                )
            })
            .collect();
        for (test_asset_name, test_asset_info) in test_assets_large.clone().into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(&mut deps.storage, test_asset_name.into(), insert)?;
        }
        let msg = query_asset_list_msg("".to_string(), 42);
        let res: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;
        assert!(res.assets.len() == 25 as usize);

        // Assert that despite 30 entries the returned data is capped at the `MAX_LIMIT` of 25 results
        assert!(res.assets.len() == 25 as usize);
        Ok(())
    }
    #[test]
    fn test_query_contract_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_contract_entry_and_string(vec![("foo", "foo", "foo")]);
        for (key, new_address) in to_add.into_iter() {
            let addr = deps.as_ref().api.addr_validate(&new_address)?;
            let insert = |_| -> StdResult<Addr> { Ok(addr) };
            CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
        }
        // create second entry
        let to_add1 = create_contract_entry_and_string(vec![("bar", "bar", "bar")]);
        for (key, new_address) in to_add1.into_iter() {
            let addr = deps.as_ref().api.addr_validate(&new_address)?;
            let insert = |_| -> StdResult<Addr> { Ok(addr) };
            CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
        }
        // create duplicate entry
        let to_add1 = create_contract_entry_and_string(vec![("bar", "bar", "bar")]);
        for (key, new_address) in to_add1.into_iter() {
            let addr = deps.as_ref().api.addr_validate(&new_address)?;
            let insert = |_| -> StdResult<Addr> { Ok(addr) };
            CONTRACT_ADDRESSES.update(&mut deps.storage, key, insert)?;
        }

        // create msgs
        let msg = QueryMsg::ContractList {
            page_token: None,
            page_size: Some(42 as u8),
        };
        let res: ContractListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        let msg = QueryMsg::ContractList {
            page_token: Some(ContractEntry {
                protocol: "bar".to_string().to_ascii_lowercase(),
                contract: "bar".to_string().to_ascii_lowercase(),
            }),
            page_size: Some(42 as u8),
        };
        let res_expect_foo: ContractListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![
                ("bar", "bar", "bar"),
                ("foo", "foo", "foo"),
            ]),
        };

        let expected_foo = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![("foo", "foo", "foo")]),
        };

        // Assert
        // Assert only returns unqiue data entries looping
        assert_that!(&res).is_equal_to(&expected);
        // Assert - sanity check for duplication
        assert_that!(&res_expect_foo).is_equal_to(&expected_foo);
        assert!(res.contracts.len() == 2 as usize);

        Ok(())
    }
    #[test]
    fn test_query_channel_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add =
            create_channel_entry_and_string(vec![("bar", "bar", "bar"), ("foo", "foo", "foo")]);
        for (key, new_channel) in to_add.into_iter() {
            // Update function for new or existing keys
            let insert = |_| -> StdResult<String> { Ok(new_channel) };
            CHANNELS.update(&mut deps.storage, key, insert)?;
        }
        // create second entry
        let to_add1 = create_channel_entry_and_string(vec![("foobar", "foobar", "foobar")]);
        for (key, new_channel) in to_add1.into_iter() {
            // Update function for new or existing keys
            let insert = |_| -> StdResult<String> { Ok(new_channel) };
            CHANNELS.update(&mut deps.storage, key, insert)?;
        }

        // create msgs
        // No token filter - should return up to `page_size` entries
        let msg = QueryMsg::ChannelList {
            page_token: None,
            page_size: Some(42 as u8),
        };
        let res_all = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Filter for entries after `Foo` - Alphabetically
        let msg = QueryMsg::ChannelList {
            page_token: Some(ChannelEntry {
                connected_chain: "foo".to_string(),
                protocol: "foo".to_string(),
            }),
            page_size: Some(42 as u8),
        };
        let res_foobar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Return first entry - Alphabetically
        let msg = QueryMsg::ChannelList {
            page_token: None,
            page_size: Some(1 as u8),
        };
        let res_bar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test

        // Return all
        let expected_all = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![
                ("bar", "bar", "bar"),
                ("foo", "foo", "foo"),
                ("foobar", "foobar", "foobar"),
            ]),
        };
        // Filter from `Foo`
        let expected_foobar = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![("foobar", "foobar", "foobar")]),
        };
        // Return first entry (alphabetically)
        let expected_bar = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![("bar", "bar", "bar")]),
        };
        // Assert
        assert_that!(&res_all).is_equal_to(expected_all);
        assert_that!(&res_foobar).is_equal_to(expected_foobar);
        assert_that!(&res_bar).is_equal_to(expected_bar);
        assert!(res_all.channels.len() == 3 as usize);

        Ok(())
    }

    #[test]
    fn test_query_registered_dexes() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // Create test data
        let to_add: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        for _dex in to_add.clone() {
            let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
                for _dex in to_add.clone() {
                    if !dexes.contains(&_dex) {
                        dexes.push(_dex.to_ascii_lowercase());
                    }
                }
                Ok(dexes)
            };
            REGISTERED_DEXES.update(&mut deps.storage, register_dex)?;
        }
        // create duplicate entry
        let to_add: Vec<String> = vec!["foo".to_string(), "foo".to_string()];
        for _dex in to_add.clone() {
            let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
                for _dex in to_add.clone() {
                    if !dexes.contains(&_dex) {
                        dexes.push(_dex.to_ascii_lowercase());
                    }
                }
                Ok(dexes)
            };
            REGISTERED_DEXES.update(&mut deps.storage, register_dex)?;
        }
        // create msg
        let msg = QueryMsg::RegisteredDexes {};
        // deserialize response
        let res: RegisteredDexesResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // comparisons
        let expected = RegisteredDexesResponse {
            dexes: vec!["foo".to_string(), "bar".to_string()],
        };
        // tests
        assert_that!(&res).is_equal_to(expected);
        // assert no duplication
        assert!(res.dexes.len() == 2 as usize);
        assert!(res.dexes[0] == ("foo"));
        assert!(res.dexes[1] == ("bar"));
        Ok(())
    }
    #[test]
    fn test_query_pools() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;
        // create DexAssetPairing
        let dex = DexAssetPairing::new("foo", "foo", "foo");
        let _pool_ref = create_option_pool_ref(42, "foo", api);
        let insert = |pool_ref: Option<Vec<PoolReference>>| -> StdResult<_> {
            let _pool_ref = pool_ref.unwrap_or_default();
            Ok(_pool_ref)
        };
        ASSET_PAIRINGS.update(&mut deps.storage, dex, insert)?;

        // create msg
        let msg = QueryMsg::Pools {
            keys: vec![DexAssetPairing::new("foo", "foo", "foo")],
        };
        let res: PoolsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;
        //comparisons
        let expected = ASSET_PAIRINGS
            .load(&deps.storage, DexAssetPairing::new("foo", "foo", "foo"))
            .unwrap();
        let expected = PoolsResponse {
            pools: vec![(DexAssetPairing::new("foo", "foo", "foo"), expected)],
        };
        // assert
        assert_eq!(&res, &expected);
        Ok(())
    }

    #[test]
    fn test_query_pool_id_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;
        // create First pool entry
        let dex_bar = DexAssetPairing::new("bar", "bar", "bar");
        let _pool_ref_bar = create_option_pool_ref(42, "bar", api);
        let insert = |pool_ref_bar: Option<Vec<PoolReference>>| -> StdResult<_> {
            let _pool_ref_bar = pool_ref_bar.unwrap_or_default();
            Ok(_pool_ref_bar)
        };
        ASSET_PAIRINGS.update(&mut deps.storage, dex_bar, insert)?;

        // create second pool entry
        let dex_foo = DexAssetPairing::new("foo", "foo", "foo");
        let _pool_ref_foo = create_option_pool_ref(69, "foo", api);
        let insert = |pool_ref_foo: Option<Vec<PoolReference>>| -> StdResult<_> {
            let _pool_ref_foo = pool_ref_foo.unwrap_or_default();
            Ok(_pool_ref_foo)
        };
        ASSET_PAIRINGS.update(&mut deps.storage, dex_foo, insert)?;

        // create duplicate pool entry
        let dex_foo = DexAssetPairing::new("foo", "foo", "foo");
        let _pool_ref_foo = create_option_pool_ref(69, "foo", api);
        let insert = |pool_ref_foo: Option<Vec<PoolReference>>| -> StdResult<_> {
            let _pool_ref_foo = pool_ref_foo.unwrap_or_default();
            Ok(_pool_ref_foo)
        };
        ASSET_PAIRINGS.update(&mut deps.storage, dex_foo, insert)?;
        // create msgs bar/ foo / foo using `page_token` as filter
        let msg_bar = QueryMsg::PoolList {
            filter: Some(AssetPairingFilter {
                asset_pair: Some(("bar".to_string(), "bar".to_string())),
                dex: None,
            }),
            page_token: None,
            page_size: None,
        };
        let res_bar: PoolIdListResponse = from_binary(&query_helper(deps.as_ref(), msg_bar)?)?;

        let msg_foo = QueryMsg::PoolList {
            filter: Some(AssetPairingFilter {
                asset_pair: Some(("foo".to_string(), "foo".to_string())),
                dex: None,
            }),
            page_token: None,
            page_size: Some(42),
        };
        let res_foo: PoolIdListResponse = from_binary(&query_helper(deps.as_ref(), msg_foo)?)?;

        let msg_foo_using_page_token = QueryMsg::PoolList {
            filter: Some(AssetPairingFilter {
                asset_pair: None,
                dex: None,
            }),
            page_token: Some(DexAssetPairing::new("bar", "bar", "bar")),
            page_size: Some(42),
        };
        let res_foo_using_page_token: PoolIdListResponse =
            from_binary(&query_helper(deps.as_ref(), msg_foo_using_page_token)?)?;

        // create comparisons - bar / foo / all
        let expected_bar = ASSET_PAIRINGS
            .load(&deps.storage, DexAssetPairing::new("bar", "bar", "bar"))
            .unwrap();
        let expected_bar = PoolIdListResponse {
            pools: vec![(DexAssetPairing::new("bar", "bar", "bar"), expected_bar)],
        };

        let expected_foo = ASSET_PAIRINGS
            .load(&deps.storage, DexAssetPairing::new("foo", "foo", "foo"))
            .unwrap();
        let expected_foo = PoolIdListResponse {
            pools: vec![(DexAssetPairing::new("foo", "foo", "foo"), expected_foo)],
        };
        let expected_all_bar = ASSET_PAIRINGS
            .load(&deps.storage, DexAssetPairing::new("bar", "bar", "bar"))
            .unwrap();
        let expected_all_foo = ASSET_PAIRINGS
            .load(&deps.storage, DexAssetPairing::new("foo", "foo", "foo"))
            .unwrap();
        let expected_all = PoolIdListResponse {
            pools: vec![
                (DexAssetPairing::new("bar", "bar", "bar"), expected_all_bar),
                (DexAssetPairing::new("foo", "foo", "foo"), expected_all_foo),
            ],
        };
        // comparison all
        let msg_all = QueryMsg::PoolList {
            filter: None,
            page_token: None,
            page_size: Some(42),
        };
        let res_all: PoolIdListResponse = from_binary(&query_helper(deps.as_ref(), msg_all)?)?;

        // assert
        assert_eq!(&res_bar, &expected_bar);
        assert_eq!(&res_foo, &expected_foo);
        assert!(&res_foo.pools.len() == &1usize);
        assert_eq!(&res_foo_using_page_token, &expected_foo);
        assert_eq!(&res_all, &expected_all);
        Ok(())
    }
    #[test]
    fn test_query_pool_metadata() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        // create metadata entries
        let bar_key = UniquePoolId::new(42);
        let bar_metadata = create_pool_metadata("bar", "ETH", "BTC");
        let insert = |_| -> StdResult<PoolMetadata> { Ok(bar_metadata) };
        POOL_METADATA.update(&mut deps.storage, bar_key, insert)?;

        let foo_key = UniquePoolId::new(69);
        let foo_metadata = create_pool_metadata("foo", "JUNO", "ATOM");
        let insert = |_| -> StdResult<PoolMetadata> { Ok(foo_metadata) };
        POOL_METADATA.update(&mut deps.storage, foo_key, insert)?;

        // create msgs
        let msg_bar = QueryMsg::PoolMetadatas {
            keys: vec![UniquePoolId::new(42)],
        };
        let res_bar: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_bar)?)?;

        let msg_foo = QueryMsg::PoolMetadatas {
            keys: vec![UniquePoolId::new(69)],
        };
        let res_foo: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_foo)?)?;

        // create comparisons
        let expected_bar = PoolMetadatasResponse {
            metadatas: vec![(
                UniquePoolId::new(42),
                PoolMetadata::new(
                    "bar",
                    abstract_os::objects::PoolType::Stable,
                    &vec!["ETH".to_string(), "BTC".to_string()],
                ),
            )],
        };
        let expected_foo = PoolMetadatasResponse {
            metadatas: vec![(
                UniquePoolId::new(69),
                PoolMetadata::new(
                    "foo",
                    abstract_os::objects::PoolType::Stable,
                    &vec!["JUNO".to_string(), "ATOM".to_string()],
                ),
            )],
        };

        assert_eq!(&res_bar, &expected_bar);
        assert_eq!(&res_foo, &expected_foo);

        // assert

        Ok(())
    }
}
