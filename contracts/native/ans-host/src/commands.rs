use cosmwasm_std::{Addr, DepsMut, Empty, MessageInfo, Response, StdResult};
use cosmwasm_std::{Env, StdError, Storage};
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use abstract_os::ans_host::{AssetPair, ExecuteMsg};
use abstract_os::ans_host::state::*;
use abstract_os::dex::DexName;
use abstract_os::objects::{
    DexAssetPairing, UncheckedChannelEntry, UncheckedContractEntry, UniquePoolId,
};
use abstract_os::objects::pool_id::{PoolId, UncheckedPoolId};
use abstract_os::objects::pool_info::PoolMetadata;
use abstract_os::objects::pool_reference::PoolReference;

use crate::contract::AnsHostResult;
use crate::error::AnsHostError;
use crate::error::AnsHostError::InvalidAssetCount;

const MIN_POOL_ASSETS: usize = 2;
const MAX_POOL_ASSETS: usize = 5;

/// Handles the common base execute messages
pub fn handle_message(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    message: ExecuteMsg,
) -> AnsHostResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateContractAddresses { to_add, to_remove } => {
            update_contract_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateAssetAddresses { to_add, to_remove } => {
            update_asset_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateChannels { to_add, to_remove } => {
            update_channels(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateDexes { to_add, to_remove } => update_dex_registry(deps, info, to_add, to_remove),
        ExecuteMsg::UpdatePools { to_add, to_remove } => {
            update_pools(deps, info, to_add, to_remove)
        }
    }
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Adds, updates or removes provided addresses.
pub fn update_contract_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedContractEntry, String)>,
    to_remove: Vec<UncheckedContractEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_address) in to_add.into_iter() {
        let key = key.check();
        // validate addr
        // let addr = deps.as_ref().api.addr_validate(&new_address)?;
        // Update function for new or existing keys
        let insert = |_| -> StdResult<Addr> { Ok(Addr::unchecked(new_address)) };
        CONTRACT_ADDRESSES.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CONTRACT_ADDRESSES.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_asset_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, AssetInfoUnchecked)>,
    to_remove: Vec<String>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_asset) in to_add.into_iter() {
        // Update function for new or existing keys
        let extension = deps.api;
        let insert = |_| -> StdResult<AssetInfo> {
            // use own check, cw_asset otherwise changes cases to lowercase
            new_asset.check(extension, None)
        };
        ASSET_ADDRESSES.update(deps.storage, name.into(), insert)?;
    }

    for name in to_remove {
        ASSET_ADDRESSES.remove(deps.storage, name.into());
    }

    Ok(Response::new().add_attribute("action", "updated asset addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_channels(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedChannelEntry, String)>,
    to_remove: Vec<UncheckedChannelEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_channel) in to_add.into_iter() {
        let key = key.check();
        // Update function for new or existing keys
        let insert = |_| -> StdResult<String> { Ok(new_channel) };
        CHANNELS.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CHANNELS.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Updates the dex registry with additions and removals
fn update_dex_registry(deps: DepsMut, info: MessageInfo, to_add: Vec<String>, to_remove: Vec<String>) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if !to_add.is_empty() {
        let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
            for dex in to_add {
                if !dexes.contains(&dex) {
                    dexes.push(dex);
                }
            }
            Ok(dexes)
        };

        REGISTERED_DEXES.update(deps.storage, register_dex)?;
    }

    if !to_remove.is_empty() {
        let deregister_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
            for dex in to_remove {
                dexes.retain(|x| x != &dex);
            }
            Ok(dexes)
        };
        REGISTERED_DEXES.update(deps.storage, deregister_dex)?;
    }

    Ok(Response::new().add_attribute("action", "update dex registry"))
}

fn update_pools(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<(UncheckedPoolId, PoolMetadata)>,
    to_remove: Vec<UniquePoolId>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut next_unique_pool_id = CONFIG.load(deps.storage)?.next_unique_pool_id;

    // only load dexes if necessary
    let registered_dexes = if to_add.is_empty() {
        vec![]
    } else {
        REGISTERED_DEXES.load(deps.storage)?
    };

    for (pool_id, pool_metadata) in to_add.into_iter() {
        let pool_id = pool_id.check(deps.api)?;

        let assets = &pool_metadata.assets;
        validate_pool_assets(assets)?;

        let dex = pool_metadata.dex.clone();
        if !registered_dexes.contains(&dex) {
            return Err(AnsHostError::UnregisteredDex { dex });
        }

        // Register each pair of assets as a pairing and link it to the pool id
        register_pool_pairings(deps.storage, next_unique_pool_id, pool_id, assets, &dex)?;

        POOL_METADATA.save(deps.storage, next_unique_pool_id, &pool_metadata)?;

        // Increment the unique pool id for the next pool
        next_unique_pool_id.increment();
    }

    for pool_id_to_remove in to_remove {
        // load the pool metadata
        let pool_metadata = POOL_METADATA.load(deps.storage, pool_id_to_remove)?;

        remove_pool_pairings(
            deps.storage,
            pool_id_to_remove,
            &pool_metadata.dex,
            &pool_metadata.assets,
        )?;

        // remove the pool metadata
        POOL_METADATA.remove(deps.storage, pool_id_to_remove);
    }

    // Save the next unique pool id
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.next_unique_pool_id = next_unique_pool_id;
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "updated pools"))
}

/// Execute an action on every asset pairing in the list of assets
/// Example: assets: [A, B, C] -> [A, B], [A, C], [B, C]
fn exec_on_asset_pairings<T, A, E>(assets: &[String], mut action: A) -> StdResult<()>
    where
        A: FnMut(AssetPair) -> Result<T, E>,
        StdError: From<E>,
{
    for (i, asset_x) in assets.iter().enumerate() {
        for (j, asset_y) in assets.iter().enumerate() {
            // Skip self-pairings
            if i == j || asset_x == asset_y {
                continue;
            }
            let pair: AssetPair = (asset_x.clone(), asset_y.clone());
            action(pair)?;
        }
    }
    Ok(())
}

fn register_pool_pairings(
    storage: &mut dyn Storage,
    next_pool_id: UniquePoolId,
    pool_id: PoolId,
    assets: &[String],
    dex: &DexName,
) -> StdResult<()> {
    let register_pairing = |(asset_x, asset_y): AssetPair| {
        let key = DexAssetPairing::new(&asset_x, &asset_y, dex);

        let compound_pool_id = PoolReference {
            id: next_pool_id,
            pool_id: pool_id.clone(),
        };

        register_asset_pairing(storage, key, compound_pool_id)
    };

    exec_on_asset_pairings(assets, register_pairing)
}

/// Register an asset pairing to its pool id
/// We ignore any duplicates, which is why we don't check for them
fn register_asset_pairing(
    storage: &mut dyn Storage,
    pair: DexAssetPairing,
    compound_pool_id: PoolReference,
) -> Result<Vec<PoolReference>, StdError> {
    let insert = |ids: Option<Vec<PoolReference>>| -> StdResult<_> {
        let mut ids = ids.unwrap_or_default();

        ids.push(compound_pool_id);
        Ok(ids)
    };

    ASSET_PAIRINGS.update(storage, pair, insert)
}

/// Remove the unique_pool_id (which is getting removed) from the list of pool ids for each asset pairing
fn remove_pool_pairings(
    storage: &mut dyn Storage,
    pool_id_to_remove: UniquePoolId,
    dex: &DexName,
    assets: &[String],
) -> StdResult<()> {
    let remove_pairing_action = |(asset_x, asset_y): AssetPair| -> Result<(), StdError> {
        let key = DexAssetPairing::new(&asset_x, &asset_y, dex);

        // Action to remove the pool id from the list of pool ids for the asset pairing
        let remove_pool_id_action = |ids: Option<Vec<PoolReference>>| -> StdResult<_> {
            let mut ids = ids.unwrap_or_default();
            ids.retain(|id| id.id != pool_id_to_remove);
            Ok(ids)
        };

        let remaining_ids = ASSET_PAIRINGS.update(storage, key.clone(), remove_pool_id_action)?;

        // If there are no remaining pools, remove the asset pair from the map
        if remaining_ids.is_empty() {
            ASSET_PAIRINGS.remove(storage, key);
        }
        Ok(())
    };

    exec_on_asset_pairings(assets, remove_pairing_action)
}

fn validate_pool_assets(assets: &[String]) -> Result<(), AnsHostError> {
    if assets.len() < MIN_POOL_ASSETS || assets.len() > MAX_POOL_ASSETS {
        return Err(InvalidAssetCount {
            min: MIN_POOL_ASSETS,
            max: MAX_POOL_ASSETS,
            provided: assets.len(),
        });
    }
    Ok(())
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> AnsHostResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use std::str::FromStr;

    use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Order, OwnedDeps, Response};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cw_controllers::AdminError;
    use cw_storage_plus::{KeyDeserialize, Map, PrimaryKey};
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use abstract_os::ans_host::InstantiateMsg;
    use abstract_os::objects::ans_host::AnsHost;

    use crate::contract;
    use crate::contract::{AnsHostResult, instantiate};
    use crate::error::AnsHostError;

    use super::*;

    type AnsHostTestResult = Result<(), AnsHostError>;

    const TEST_CREATOR: &str = "creator";

    fn mock_init(mut deps: DepsMut) -> AnsHostResult {
        let info = mock_info(TEST_CREATOR, &[]);

        instantiate(
            deps.branch(),
            mock_env(),
            info,
            InstantiateMsg {},
        )
    }

    mod update_dexes {
        use cosmwasm_std::OwnedDeps;
        use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};

        use super::*;

        #[test]
        fn register_dex() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        /// Registering multiple dexes should work
        #[test]
        fn register_dex_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;
            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;


            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        #[test]
        fn duplicate_in_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone(), new_dex.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            // ONly one dex should be registered
            assert_expected_dexes(&deps, vec![new_dex]);

            Ok(())
        }

        #[test]
        fn register_and_deregister_dex_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![new_dex.clone()],
                to_remove: vec![new_dex.clone()],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, vec![]);

            Ok(())
        }

        #[test]
        fn register_multiple_dexes() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_dexes = vec!["test_dex".to_string(), "test_dex_2".to_string()];

            let msg = ExecuteMsg::UpdateDexes {
                to_add: new_dexes.clone(),
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            assert_expected_dexes(&deps, new_dexes);

            Ok(())
        }

        #[test]
        fn remove_nonexistent_dex() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let missing_dex = "test_dex".to_string();

            let msg = ExecuteMsg::UpdateDexes {
                to_add: vec![],
                to_remove: vec![missing_dex.clone()],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            let expected_dexes: Vec<String> = vec![];

            assert_expected_dexes(&deps, expected_dexes);

            Ok(())
        }

        fn assert_expected_dexes(deps: &OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>, expected_dexes: Vec<String>) {
            let actual_dexes = REGISTERED_DEXES.load(&deps.storage).unwrap();

            assert_eq!(actual_dexes, expected_dexes);
        }
    }

    mod update_contract_addresses {
        use cosmwasm_std::{Order, OwnedDeps};
        use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};

        use abstract_os::objects::ContractEntry;

        use super::*;

        fn contract_entry(provider: &str, name: &str) -> UncheckedContractEntry {
            UncheckedContractEntry {
                protocol: provider.to_string(),
                contract: name.to_string(),
            }
        }

        fn contract_address_map_entry(provider: &str, name: &str, address: &str) -> (UncheckedContractEntry, String) {
            (contract_entry(provider, name), address.to_string())
        }

        fn mock_contract_map_entry() -> (UncheckedContractEntry, String) {
            contract_address_map_entry("test_provider", "test_contract", "test_address")
        }

        fn update_contract_addresses_msg_builder(to_add: Vec<(UncheckedContractEntry, String)>,
                                                 to_remove: Vec<UncheckedContractEntry>, ) -> ExecuteMsg {
            ExecuteMsg::UpdateContractAddresses {
                to_add,
                to_remove,
            }
        }

        fn from_checked_entry(checked: (ContractEntry, Addr)) -> (UncheckedContractEntry, String) {
            let key = checked.0;
            let value = checked.1;
            (UncheckedContractEntry {
                protocol: key.protocol,
                contract: key.contract,
            }, value.into())
        }

        fn setup_map_tester<'a>() -> CwMapTester<'a, ContractEntry, Addr, UncheckedContractEntry, String> {
            let info = mock_info(TEST_CREATOR, &[]);

            let tester = CwMapTester::new(info, CONTRACT_ADDRESSES, update_contract_addresses_msg_builder, mock_contract_map_entry, from_checked_entry);

            tester
        }

        #[test]
        fn add_contract_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one(&mut deps)
        }

        #[test]
        fn add_contract_address_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_one_twice(&mut deps)
        }

        #[test]
        fn add_contract_address_twice_in_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_two_same(&mut deps)
        }

        #[test]
        fn add_and_remove_contract_address_same_msg() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_add_and_remove_same(&mut deps)
        }

        #[test]
        fn remove_non_existent_contract_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let mut map_tester = setup_map_tester();
            map_tester.test_remove_nonexistent(&mut deps)
        }

        #[test]
        fn add_multiple_contract_addresses() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_entry_1 = contract_address_map_entry("test_provider", "test_contract", "test_address");
            let new_entry_2 = contract_address_map_entry("test_provider_2", "test_contract_2", "test_address_2");

            let msg = ExecuteMsg::UpdateContractAddresses {
                to_add: vec![new_entry_1.clone(), new_entry_2.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            map_tester.assert_expected_entries(&deps.storage, vec![new_entry_1, new_entry_2]);

            Ok(())
        }

        #[test]
        fn add_multiple_contract_addresses_and_deregister_one() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let mut map_tester = setup_map_tester();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_entry_1 = contract_address_map_entry("test_provider", "test_contract", "test_address");
            let new_entry_2 = contract_address_map_entry("test_provider_2", "test_contract_2", "test_address_2");

            let msg = ExecuteMsg::UpdateContractAddresses {
                to_add: vec![new_entry_1.clone(), new_entry_2.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;

            let new_entry_3 = contract_address_map_entry("test_provider_3", "test_contract_3", "test_address_3");

            let msg = ExecuteMsg::UpdateContractAddresses {
                to_add: vec![new_entry_3.clone()],
                to_remove: vec![new_entry_1.clone().0],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            map_tester.assert_expected_entries(&deps.storage, vec![new_entry_2, new_entry_3]);

            Ok(())
        }
    }

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    struct CwMapTester<'a, K, V, UncheckedK, UncheckedV> {
        info: MessageInfo,
        map: Map<'a, K, V>,
        msg_builder: fn(to_add: Vec<(UncheckedK, UncheckedV)>, to_remove: Vec<UncheckedK>) -> ExecuteMsg,
        mock_entry_builder: fn() -> (UncheckedK, UncheckedV),
        from_checked_entry: fn((K, V)) -> (UncheckedK, UncheckedV),
    }

    impl<'a, K, V, UncheckedK, UncheckedV> CwMapTester<'a, K, V, UncheckedK, UncheckedV>
        where V: Serialize + DeserializeOwned + Clone + Debug,
              K: PrimaryKey<'a> + KeyDeserialize + Debug,
              (<K as KeyDeserialize>::Output, V): PartialEq<(K, V)>,
              K::Output: 'static,
              UncheckedK: From<<K as KeyDeserialize>::Output> + Clone + PartialEq + Debug + Ord,
              UncheckedV: From<V> + Clone + PartialEq + Debug + Ord
        , <K as KeyDeserialize>::Output: Debug
    {
        fn new(info: MessageInfo, map: Map<'a, K, V>,
               msg_builder: fn(to_add: Vec<(UncheckedK, UncheckedV)>, to_remove: Vec<UncheckedK>) -> ExecuteMsg,
               mock_entry_builder: fn() -> (UncheckedK, UncheckedV), from_checked_entry: fn((K, V)) -> (UncheckedK, UncheckedV),
        ) -> Self {
            Self {
                info,
                map,
                msg_builder,
                mock_entry_builder,
                from_checked_entry,
            }
        }

        fn msg_builder(&self, to_add: Vec<(UncheckedK, UncheckedV)>, to_remove: Vec<UncheckedK>) -> ExecuteMsg {
            (self.msg_builder)(to_add, to_remove)
        }

        fn mock_entry_builder(&self) -> (UncheckedK, UncheckedV) {
            (self.mock_entry_builder)()
        }

        fn determine_expected(&self, to_add: &Vec<(UncheckedK, UncheckedV)>, to_remove: &Vec<UncheckedK>) -> Vec<(UncheckedK, UncheckedV)> {
            let mut expected = to_add.clone();
            expected.retain(|(k, _)| !to_remove.contains(k));
            expected.sort();
            expected.dedup();
            expected
        }

        fn assert_expected_entries<'c>(&self, storage: &'c dyn Storage, expected: Vec<(UncheckedK, UncheckedV)>) {
            let res: Result<Vec<(K::Output, V)>, _> = self.map.range(storage, None, None, Order::Ascending).collect();

            let actual = res.unwrap().into_iter().map(|(k, v)| (k.into(), v.into())).collect::<Vec<_>>();

            // Sort, like map entries
            let mut expected = expected.clone();
            expected.sort();

            assert_eq!(actual, expected)
        }

        fn test_add_one(&mut self, deps: &mut MockDeps) -> AnsHostTestResult {
            let entry = self.mock_entry_builder();

            let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry];
            let to_remove: Vec<UncheckedK> = vec![];
            let msg = self.msg_builder(to_add.clone(), to_remove.clone());

            let expected = self.determine_expected(&to_add, &to_remove);

            contract::execute(deps.as_mut(), mock_env(), self.info.clone(), msg)?;

            self.assert_expected_entries(&deps.storage, expected);

            Ok(())
        }

        fn test_add_one_twice(&mut self, deps: &mut MockDeps) -> AnsHostTestResult {
            self.test_add_one(deps)?;
            self.test_add_one(deps)
        }

        fn test_add_two_same(&mut self, deps: &mut MockDeps) -> AnsHostTestResult {
            let entry = self.mock_entry_builder();

            let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone(), entry.clone()];
            let to_remove: Vec<UncheckedK> = vec![];
            let msg = self.msg_builder(to_add.clone(), to_remove.clone());

            let expected: Vec<(UncheckedK, UncheckedV)> = self.determine_expected(&to_add, &to_remove);

            contract::execute(deps.as_mut(), mock_env(), self.info.clone(), msg)?;

            self.assert_expected_entries(&deps.storage, expected);

            Ok(())
        }

        fn test_add_and_remove_same(&mut self, deps: &mut MockDeps) -> AnsHostTestResult {
            let entry = self.mock_entry_builder();

            let to_add: Vec<(UncheckedK, UncheckedV)> = vec![entry.clone()];
            let to_remove: Vec<UncheckedK> = vec![entry.0];
            let msg = self.msg_builder(to_add.clone(), to_remove.clone());

            let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

            contract::execute(deps.as_mut(), mock_env(), self.info.clone(), msg)?;

            self.assert_expected_entries(&deps.storage, expected);

            Ok(())
        }

        fn test_remove_nonexistent(&mut self, deps: &mut MockDeps) -> AnsHostTestResult {
            let entry = self.mock_entry_builder();

            let to_add: Vec<(UncheckedK, UncheckedV)> = vec![];
            let to_remove: Vec<UncheckedK> = vec![entry.0];
            let msg = self.msg_builder(to_add.clone(), to_remove.clone());

            let expected: Vec<(UncheckedK, UncheckedV)> = vec![];

            contract::execute(deps.as_mut(), mock_env(), self.info.clone(), msg)?;

            self.assert_expected_entries(&deps.storage, expected);

            Ok(())
        }
    }


    mod update_asset_addresses {
        use super::*;

        fn mock_asset_entry() -> (String, AssetInfoUnchecked) {
            let name = "test";
            let info = AssetInfoUnchecked::native("utest".to_string());

            (name.into(), info)
        }

        #[test]
        fn add_asset_address() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_entry = mock_asset_entry();

            let msg = ExecuteMsg::UpdateAssetAddresses {
                to_add: vec![new_entry.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            // assert_expected_asset_addresses(&deps, vec![new_entry]);

            Ok(())
        }

        #[test]
        fn add_asset_address_twice() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_entry = mock_asset_entry();

            let msg = ExecuteMsg::UpdateAssetAddresses {
                to_add: vec![new_entry.clone(), new_entry.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;
            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            // assert_expected_asset_addresses(&deps, vec![new_entry]);

            Ok(())
        }

        #[test]
        fn add_multiple_asset_addresses() -> AnsHostTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let info = mock_info(TEST_CREATOR, &[]);
            let new_entry_1 = mock_asset_entry();
            let new_entry_2 = mock_asset_entry();

            let msg = ExecuteMsg::UpdateAssetAddresses {
                to_add: vec![new_entry_1.clone(), new_entry_2.clone()],
                to_remove: vec![],
            };

            let res = contract::execute(deps.as_mut(), mock_env(), info, msg)?;

            // assert_expected_asset_addresses(&deps, vec![new_entry_1, new_entry_2]);

            Ok(())
        }
    }

    mod validate_pool_assets {
        use super::*;

        #[test]
        fn too_few() {
            let result = validate_pool_assets(&[]).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 0,
                }
            );

            let result = validate_pool_assets(&["a".to_string()]).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 1,
                }
            );
        }

        #[test]
        fn valid_amounts() {
            let assets = vec!["a".to_string(), "b".to_string()];
            let res = validate_pool_assets(&assets);
            assert!(res.is_ok());

            let assets: Vec<String> = vec!["a", "b", "c", "d", "e"].iter().map(|s| s.to_string()).collect();
            let res = validate_pool_assets(&assets);
            assert!(res.is_ok());
        }

        #[test]
        fn too_many() {
            let assets: Vec<String> = vec!["a", "b", "c", "d", "e", "f"].iter().map(|s| s.to_string()).collect();
            let result = validate_pool_assets(&assets).unwrap_err();
            assert_eq!(
                result,
                InvalidAssetCount {
                    min: MIN_POOL_ASSETS,
                    max: MAX_POOL_ASSETS,
                    provided: 6,
                }
            );
        }
    }
}
