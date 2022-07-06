use cosmwasm_std::{
    entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::queries::{handle_config_query, handle_module_info_query, handle_os_info_query};
use crate::validators::{validate_description, validate_link, validate_name_or_gov_type};
use crate::{commands::*, error::ManagerError, queries};
use abstract_os::manager::state::{Config, OsInfo, ADMIN, CONFIG, INFO, ROOT, STATUS};
use abstract_os::MANAGER;
use abstract_os::{
    manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    objects::module::*,
    proxy::state::OS_ID,
};
use cw2::set_contract_version;

pub type ManagerResult = Result<Response, ManagerError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MIN_DESC_LENGTH: usize = 4;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
pub(crate) const MIN_LINK_LENGTH: usize = 12;
pub(crate) const MAX_LINK_LENGTH: usize = 128;
pub(crate) const MIN_TITLE_LENGTH: usize = 4;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    // let version: Version = CONTRACT_VERSION.parse()?;
    // let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;
    // if storage_version < version {
    //     set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    // }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    let subscription_address = if let Some(addr) = msg.subscription_address {
        deps.api.addr_validate(&addr)?
    } else if msg.os_id == 0 {
        Addr::unchecked("".to_string())
    } else {
        return Err(ManagerError::NoSubscriptionAddrProvided {});
    };

    OS_ID.save(deps.storage, &msg.os_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
            module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
            subscription_address,
        },
    )?;

    // Verify info
    validate_description(&msg.description)?;
    validate_link(&msg.link)?;
    validate_name_or_gov_type(&msg.os_name)?;

    let os_info = OsInfo {
        name: msg.os_name,
        governance_type: msg.governance_type,
        chain_id: msg.chain_id,
        description: msg.description,
        link: msg.link,
    };

    INFO.save(deps.storage, &os_info)?;
    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    STATUS.save(deps.storage, &true)?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::SuspendOs { new_status } => update_os_status(deps, info, new_status),
        msg => {
            // Block actions if user is not subscribed
            let is_subscribed = STATUS.load(deps.storage)?;
            if !is_subscribed {
                return Err(ManagerError::NotSubscribed {});
            }

            match msg {
                ExecuteMsg::SetAdmin {
                    admin,
                    governance_type,
                } => set_admin_and_gov_type(deps, info, admin, governance_type),
                ExecuteMsg::UpdateConfig { vc_addr, root } => {
                    execute_update_config(deps, info, vc_addr, root)
                }
                ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
                    // Only Admin can call this method
                    // Todo: Admin is currently defaulted to Os Factory.
                    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
                    update_module_addresses(deps, to_add, to_remove)
                }
                ExecuteMsg::CreateModule { module, init_msg } => {
                    create_module(deps, info, env, module, init_msg)
                }
                ExecuteMsg::RegisterModule {
                    module,
                    module_addr,
                } => register_module(deps, info, env, module, module_addr),
                ExecuteMsg::ExecOnModule {
                    module_name,
                    exec_msg,
                } => exec_on_module(deps, info, module_name, exec_msg),
                ExecuteMsg::Upgrade {
                    module,
                    migrate_msg,
                } => _upgrade_module(deps, env, info, module, migrate_msg),
                ExecuteMsg::RemoveModule { module_name } => remove_module(deps, info, module_name),
                ExecuteMsg::UpdateInfo {
                    os_name,
                    description,
                    link,
                } => update_info(deps, info, os_name, description, link),
                _ => panic!(),
            }
        }
    }
}

fn _upgrade_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: Module,
    migrate_msg: Option<Binary>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    match module.kind {
        ModuleKind::API => replace_api(deps, module.info),
        _ => match migrate_msg {
            Some(msg) => migrate_module(deps, env, module.info, msg),
            None => Err(ManagerError::MsgRequired {}),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ModuleVersions { names } => {
            queries::handle_contract_versions_query(deps, env, names)
        }
        QueryMsg::ModuleAddresses { names } => {
            queries::handle_module_address_query(deps, env, names)
        }
        QueryMsg::ModuleInfos {
            last_module_name,
            iter_limit,
        } => handle_module_info_query(deps, last_module_name, iter_limit),
        QueryMsg::Info {} => handle_os_info_query(deps),
        QueryMsg::Config {} => handle_config_query(deps),
    }
}
