use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use crate::commands::*;
use crate::error::MemoryError;
use crate::queries;
use abstract_os::memory::state::ADMIN;
use abstract_os::memory::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

pub type MemoryResult = Result<Response, MemoryError>;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
use abstract_os::MEMORY;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> MemoryResult {
    set_contract_version(deps.storage, MEMORY, CONTRACT_VERSION)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> MemoryResult {
    handle_message(deps, info, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Assets { names } => queries::query_assets(deps, env, names),
        QueryMsg::Contracts { names } => queries::query_contract(deps, env, names),
        QueryMsg::AssetList {
            page_token,
            page_size,
        } => queries::query_asset_list(deps, page_token, page_size),
        QueryMsg::ContractList {
            page_token,
            page_size,
        } => queries::query_contract_list(deps, page_token, page_size),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, MEMORY, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}
