use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, Uint64,
};
use cw2::set_contract_version;
use protobuf::Message;

use crate::commands::{self, *};
use crate::error::ManagerError;
use crate::queries;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{ADMIN, NEW_MODULE, OS_ID, ROOT, VC_ADDRESS};
use pandora::manager::msg::{ConfigQueryResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use pandora::registery::MANAGER;

pub type ManagerResult = Result<Response, ManagerError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    OS_ID.save(deps.storage, &msg.os_id)?;
    VC_ADDRESS.save(deps.storage, &msg.vc_addr)?;
    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateConfig { vc_addr, root } => {
            execute_update_config(deps, info, vc_addr, root)
        }
        ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
            // Only Admin can call this method
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
            update_module_addresses(deps, to_add, to_remove)
        }
        ExecuteMsg::AddInternalDapp {
            module,
            version,
            init_msg,
        } => add_internal_dapp(deps, info, env, module, version, init_msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ManagerResult {
    match msg {
        Reply {
            id: commands::DAPP_CREATE_ID,
            result,
        } => {
            // Get address of new dApp
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
            })?;
            let module_address = res.get_contract_address();

            // Save new module details
            let module = NEW_MODULE.load(deps.storage)?;
            commands::update_module_addresses(
                deps,
                Some(vec![(module, module_address.to_string())]),
                None,
            )
        }
        _ => Err(ManagerError::UnexpectedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryVersions { names } => {
            queries::handle_contract_versions_query(deps, env, names)
        }
        QueryMsg::QueryModules { names } => {
            queries::handle_module_addresses_query(deps, env, names)
        }
        QueryMsg::QueryEnabledModules {} => queries::handle_enabled_modules_query(deps),

        QueryMsg::QueryOsConfig {} => {
            let os_id = Uint64::from(OS_ID.load(deps.storage)?);
            let root = ROOT
                .get(deps)?
                .unwrap_or_else(|| Addr::unchecked(""))
                .to_string();
            let vc_addr = VC_ADDRESS.load(deps.storage)?;

            to_binary(&ConfigQueryResponse {
                root,
                os_id,
                vc_addr,
            })
        }
    }
}
