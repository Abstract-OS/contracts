use crate::contract::{ProxyResponse, ProxyResult};
use crate::error::ProxyError;
use crate::queries;
use abstract_core::objects::{oracle::Oracle, price_source::UncheckedPriceSource, AssetEntry};
use abstract_sdk::core::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    proxy::state::{ANS_HOST, STATE},
    IBC_CLIENT,
};
use cosmwasm_std::{wasm_execute, CosmosMsg, DepsMut, Empty, MessageInfo, StdError};

const LIST_SIZE_LIMIT: usize = 15;

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
pub fn execute_module_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotAllowlisted {});
    }

    Ok(ProxyResponse::action("execute_module_action").add_messages(msgs))
}

/// Executes IBC actions forwarded by whitelisted contracts
/// Calls the messages on the IBC client (ensuring permission)
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<IbcClientMsg>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotAllowlisted {});
    }
    let manager_address = queries::get_manager(deps.storage);

    let ibc_client_address = abstract_sdk::core::manager::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address.unwrap(), IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on manager. Add it under the {IBC_CLIENT} name."
            ))
        })?;
    let client_msgs: Result<Vec<_>, _> = msgs
        .into_iter()
        .map(|execute_msg| wasm_execute(&ibc_client_address, &execute_msg, vec![]))
        .collect();

    Ok(ProxyResponse::action("execute_ibc_action").add_messages(client_msgs?))
}

/// Update the stored vault asset information
pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
    to_remove: Vec<AssetEntry>,
) -> ProxyResult {
    // Only Admin can call this method
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
    let ans_host = &ANS_HOST.load(deps.storage)?;

    let oracle = Oracle::new();
    oracle.update_assets(deps, ans_host, to_add, to_remove)?;
    Ok(ProxyResponse::action("update_proxy_assets"))
}

/// Add a contract to the whitelist
pub fn add_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if state.modules.len() >= LIST_SIZE_LIMIT {
        return Err(ProxyError::ModuleLimitReached {});
    }

    let module_addr = deps.api.addr_validate(&module)?;

    if state.modules.contains(&module_addr) {
        return Err(ProxyError::AlreadyAllowlisted(module));
    }

    // Add contract to whitelist.
    state.modules.push(module_addr);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(ProxyResponse::new("add_module", vec![("module", module)]))
}

/// Remove a contract from the whitelist
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    STATE.update(deps.storage, |mut state| {
        let module_address = deps.api.addr_validate(&module)?;

        if !state.modules.contains(&module_address) {
            return Err(ProxyError::ModuleNotAllowlisted(module.clone()));
        }
        // Remove contract from whitelist.
        state.modules.retain(|addr| *addr != module_address);
        Ok(state)
    })?;

    // Respond and note the change
    Ok(ProxyResponse::new(
        "remove_module",
        vec![("module", module)],
    ))
}

#[cfg(test)]
mod test {
    use abstract_testing::prelude::TEST_MANAGER;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::testing::{
        mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{Addr, OwnedDeps, Storage};
    use speculoos::prelude::*;

    use abstract_core::proxy::{ExecuteMsg, InstantiateMsg};

    use crate::contract::{execute, instantiate};

    use super::*;

    const TEST_MODULE: &str = "module";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_MANAGER, &[]);
        let msg = InstantiateMsg {
            account_id: 0,
            ans_host_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let _res = instantiate(deps, mock_env(), info, msg).unwrap();
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let info = mock_info(TEST_MANAGER, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    fn load_modules(storage: &dyn Storage) -> Vec<Addr> {
        STATE.load(storage).unwrap().modules
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> ProxyResult {
        execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn test_only_owner(deps: DepsMut, msg: ExecuteMsg) -> ProxyTestResult {
        let res = execute_as(deps, "not_admin", msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(ProxyError::Ownership(
                cw_ownable::OwnershipError::NotOwner {},
            ));

        Ok(())
    }

    mod add_module {
        use cosmwasm_std::testing::mock_dependencies;
        use cosmwasm_std::Addr;

        use super::*;

        #[test]
        fn only_admin_can_add_module() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            test_only_owner(deps.as_mut(), msg)
        }

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            assert_that(&actual_modules).has_length(1);
            assert_that(&actual_modules).contains(&Addr::unchecked(TEST_MODULE));
        }

        #[test]
        fn fails_adding_previously_added_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg.clone());
            assert_that(&res).is_ok();

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::AlreadyAllowlisted(TEST_MODULE.to_string()));
        }

        #[test]
        fn fails_adding_module_when_list_is_full() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let mut msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            for i in 0..LIST_SIZE_LIMIT {
                msg = ExecuteMsg::AddModule {
                    module: format!("module_{i}"),
                };
                let res = execute_as_admin(&mut deps, msg.clone());
                assert_that(&res).is_ok();
            }

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::ModuleLimitReached {});
        }
    }

    type ProxyTestResult = Result<(), ProxyError>;

    mod remove_module {
        use cosmwasm_std::testing::mock_dependencies;
        use cosmwasm_std::Addr;

        use abstract_core::proxy::state::State;

        use super::*;

        #[test]
        fn only_admin() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };
            test_only_owner(deps.as_mut(), msg)
        }

        #[test]
        fn remove_module() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![Addr::unchecked(TEST_MODULE)],
                },
            )?;

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };
            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            assert_that(&actual_modules).is_empty();

            Ok(())
        }

        #[test]
        fn fails_removing_non_existing_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::ModuleNotAllowlisted(TEST_MODULE.to_string()));
        }
    }

    mod execute_action {
        use super::*;
        use abstract_core::proxy::state::State;

        #[test]
        fn only_allowlisted_can_execute() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::ModuleAction { msgs: vec![] };

            let info = mock_info("not_allowlisted", &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::SenderNotAllowlisted {});
        }

        #[test]
        fn forwards_action() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            // stub a module
            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![Addr::unchecked(TEST_MODULE)],
                },
            )?;

            let action: CosmosMsg = wasm_execute(
                MOCK_CONTRACT_ADDR.to_string(),
                // example garbage
                &ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                    new_owner: TEST_MANAGER.to_string(),
                    expiry: None,
                }),
                vec![],
            )?
            .into();

            let msg = ExecuteMsg::ModuleAction {
                msgs: vec![action.clone()],
            };

            // execute it AS the module
            let res = execute(deps.as_mut(), mock_env(), mock_info(TEST_MODULE, &[]), msg);
            assert_that(&res).is_ok();

            let msgs = res.unwrap().messages;
            assert_that(&msgs).has_length(1);

            let msg = &msgs[0];
            assert_that(&msg.msg).is_equal_to(&action);

            Ok(())
        }
    }

    mod execute_ibc {
        use abstract_core::{manager, proxy::state::State};
        use abstract_testing::{prelude::TEST_MANAGER, MockQuerierBuilder};
        use cosmwasm_std::{to_binary, SubMsg};

        use super::*;

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());
            // whitelist creator
            STATE
                .save(
                    &mut deps.storage,
                    &State {
                        modules: vec![Addr::unchecked(TEST_MANAGER)],
                    },
                )
                .unwrap();

            let msg = ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register {
                    host_chain: "juno".into(),
                }],
            };

            let not_whitelisted_info = mock_info(TEST_MANAGER, &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = mock_info(TEST_MANAGER, &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
            // mock enabling ibc
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_MANAGER,
                    manager::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, Addr::unchecked("ibc_client_addr")),
                )
                .build();

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "ibc_client_addr".into(),
                    msg: to_binary(&abstract_core::ibc_client::ExecuteMsg::Register {
                        host_chain: "juno".into(),
                    })
                    .unwrap(),
                    funds: vec![],
                },
            )));
        }
    }

    mod update_assets {
        use super::*;

        #[test]
        fn only_admin_can_add_module() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::UpdateAssets {
                to_add: vec![],
                to_remove: vec![],
            };

            test_only_owner(deps.as_mut(), msg)
        }
    }
}
