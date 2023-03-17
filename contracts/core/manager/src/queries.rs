use abstract_sdk::os::manager::state::{OsInfo, CONFIG, INFO, OS_ID, OS_MODULES, ROOT};
use abstract_sdk::os::manager::{
    ConfigResponse, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    ModuleVersionsResponse,
};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, QueryRequest, Uint64, WasmQuery};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::Bound;
use std::collections::BTreeMap;

use crate::contract::ManagerResult;
use crate::error::ManagerError;

const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 10;

pub fn handle_module_address_query(
    deps: Deps,
    env: Env,
    ids: Vec<String>,
) -> ManagerResult<Binary> {
    let contracts = query_module_addresses(deps, &env.contract.address, &ids)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&ModuleAddressesResponse { modules: vector }).map_err(Into::into)
}

pub fn handle_contract_versions_query(
    deps: Deps,
    env: Env,
    ids: Vec<String>,
) -> ManagerResult<Binary> {
    let response = query_module_versions(deps, &env.contract.address, &ids)?;
    let versions = response.into_values().collect();
    to_binary(&ModuleVersionsResponse { versions }).map_err(Into::into)
}

pub fn handle_os_info_query(deps: Deps) -> ManagerResult<Binary> {
    let info: OsInfo = INFO.load(deps.storage)?;
    to_binary(&InfoResponse { info }).map_err(Into::into)
}

pub fn handle_config_query(deps: Deps) -> ManagerResult<Binary> {
    let os_id = Uint64::from(OS_ID.load(deps.storage)?);
    let root = ROOT
        .get(deps)?
        .unwrap_or_else(|| Addr::unchecked(""))
        .to_string();
    let config = CONFIG.load(deps.storage)?;
    to_binary(&ConfigResponse {
        root,
        os_id,
        version_control_address: config.version_control_address.to_string(),
        module_factory_address: config.module_factory_address.into_string(),
    })
    .map_err(Into::into)
}
pub fn handle_module_info_query(
    deps: Deps,
    last_module_id: Option<String>,
    limit: Option<u8>,
) -> ManagerResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_module_id.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, Addr)>, _> = OS_MODULES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let ids_and_addr = res?;
    let mut resp_vec: Vec<ManagerModuleInfo> = vec![];
    for (id, address) in ids_and_addr.into_iter() {
        let version = query_module_cw2(&deps, address.clone())?;
        resp_vec.push(ManagerModuleInfo {
            id,
            version,
            address: address.to_string(),
        })
    }

    to_binary(&ModuleInfosResponse {
        module_infos: resp_vec,
    })
    .map_err(Into::into)
}

/// RawQuery the version of an enabled module
pub fn query_module_cw2(deps: &Deps, module_addr: Addr) -> ManagerResult<ContractVersion> {
    let req = QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: module_addr.into(),
        key: CONTRACT.as_slice().into(),
    });
    deps.querier
        .query::<ContractVersion>(&req)
        .map_err(Into::into)
}

/// RawQuery the module versions of the modules part of the OS
/// Errors if not present
pub fn query_module_versions(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> ManagerResult<BTreeMap<String, ContractVersion>> {
    let addresses: BTreeMap<String, Addr> =
        query_module_addresses(deps, manager_addr, module_names)?;
    let mut module_versions: BTreeMap<String, ContractVersion> = BTreeMap::new();
    for (name, address) in addresses.into_iter() {
        let result = query_module_cw2(&deps, address)?;
        module_versions.insert(name, result);
    }
    Ok(module_versions)
}

/// RawQuery module addresses from manager
/// Errors if not present
pub fn query_module_addresses(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> ManagerResult<BTreeMap<String, Addr>> {
    let mut modules: BTreeMap<String, Addr> = BTreeMap::new();

    // Query over
    for module in module_names.iter() {
        let result: ManagerResult<Addr> = OS_MODULES
            .query(&deps.querier, manager_addr.clone(), module)?
            .ok_or_else(|| ManagerError::ModuleNotFound(module.clone()));
        // Add to map if present, skip otherwise. Allows version control to check what modules are present.
        match result {
            Ok(address) => modules.insert(module.clone(), address),
            Err(_) => None,
        };
    }
    Ok(modules)
}
