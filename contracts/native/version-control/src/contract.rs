use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_semver::Version;

use abstract_core::objects::module_version::assert_cw_contract_upgrade;
use abstract_core::version_control::Config;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    version_control::{
        state::{CONFIG, FACTORY},
        ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    VERSION_CONTROL,
};
use abstract_sdk::{execute_update_ownership, query_ownership};

use crate::commands::*;
use crate::error::VCError;
use crate::queries;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type VCResult<T = Response> = Result<T, VCError>;

#[abstract_response(VERSION_CONTROL)]
pub struct VcResponse;

pub const ABSTRACT_NAMESPACE: &str = "abstract";

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    Ok(VcResponse::action("migrate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> VCResult {
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;

    let InstantiateMsg {
        is_testnet,
        namespaces_limit,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            is_testnet,
            namespaces_limit,
        },
    )?;

    // Set up the admin as the creator of the contract
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    FACTORY.set(deps, None)?;

    Ok(VcResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::AddModules { modules } => add_modules(deps, info, modules),
        ExecuteMsg::ApproveOrRejectModules { approves, rejects } => {
            approve_or_reject_modules(deps, info, approves, rejects)
        }
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::YankModule { module } => yank_module(deps, info, module),
        ExecuteMsg::ClaimNamespaces {
            account_id,
            namespaces,
        } => claim_namespaces(deps, info, account_id, namespaces),
        ExecuteMsg::RemoveNamespaces { namespaces } => remove_namespaces(deps, info, namespaces),
        ExecuteMsg::AddAccount {
            account_id,
            account_base: base,
        } => add_account(deps, info, account_id, base),
        ExecuteMsg::UpdateNamespaceLimit { new_limit } => {
            update_namespaces_limit(deps, info, new_limit)
        }
        ExecuteMsg::SetFactory { new_factory } => set_factory(deps, info, new_factory),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(VcResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AccountBase { account_id } => {
            queries::handle_account_address_query(deps, account_id)
        }
        QueryMsg::Modules { infos } => queries::handle_modules_query(deps, infos),
        QueryMsg::Namespaces { accounts } => queries::handle_namespaces_query(deps, accounts),
        QueryMsg::Config {} => {
            let cw_ownable::Ownership { owner, .. } = cw_ownable::get_ownership(deps.storage)?;

            let factory = FACTORY.get(deps)?.unwrap();
            to_binary(&ConfigResponse {
                admin: owner.unwrap(),
                factory,
            })
        }
        QueryMsg::ModuleList {
            filter,
            start_after,
            limit,
        } => queries::handle_module_list_query(deps, start_after, limit, filter),
        QueryMsg::NamespaceList {
            filter,
            start_after,
            limit,
        } => queries::handle_namespace_list_query(deps, start_after, limit, filter),
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use crate::test_common::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;

        #[test]
        fn disallow_same_version() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: VERSION_CONTROL.to_string(),
                    from: version.to_string().parse().unwrap(),
                    to: version.to_string().parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, VERSION_CONTROL, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: VERSION_CONTROL.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version.to_string().parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.to_string(),
                    to: VERSION_CONTROL.to_string(),
                },
            ));

            Ok(())
        }

        #[test]
        fn works() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            cw2::set_contract_version(deps.as_mut().storage, VERSION_CONTROL, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
