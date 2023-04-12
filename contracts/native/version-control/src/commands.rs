use crate::contract::{VCResult, ABSTRACT_NAMESPACE};
use crate::error::VCError;
use abstract_core::objects::AccountId;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    objects::{module::ModuleInfo, module_reference::ModuleReference},
    version_control::{state::*, AccountBase},
    VERSION_CONTROL,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cw_ownable::assert_owner;

#[abstract_response(VERSION_CONTROL)]
pub struct VcResponse;

/// Add new Account to version control contract
/// Only Factory can add Account
pub fn add_account(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    account_base: AccountBase,
) -> VCResult {
    // Only Factory can add new Account
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;
    ACCOUNT_ADDRESSES.save(deps.storage, account_id, &account_base)?;

    Ok(VcResponse::new(
        "add_account",
        vec![
            ("account_id", account_id.to_string().as_str()),
            ("manager", account_base.manager.as_ref()),
            ("proxy", account_base.proxy.as_ref()),
        ],
    ))
}

/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn add_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    modules: Vec<(ModuleInfo, ModuleReference)>,
) -> VCResult {
    for (module, mod_ref) in modules {
        if MODULE_LIBRARY.has(deps.storage, &module) {
            return Err(VCError::NotUpdateableModule(module));
        }
        module.validate()?;
        mod_ref.validate(deps.as_ref())?;
        // version must be set in order to add the new version
        module.assert_version_variant()?;

        if module.namespace == ABSTRACT_NAMESPACE {
            // Only Admin can update abstract contracts
            assert_owner(deps.storage, &msg_info.sender)?;
        }
        MODULE_LIBRARY.save(deps.storage, &module, &mod_ref)?;
    }

    Ok(VcResponse::action("add_modules"))
}

/// Remove a module
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only Admin can update code-ids
    assert_owner(deps.storage, &msg_info.sender)?;
    module.assert_version_variant()?;
    if MODULE_LIBRARY.has(deps.storage, &module) {
        MODULE_LIBRARY.remove(deps.storage, &module);
    } else {
        return Err(VCError::ModuleNotFound(module));
    }

    Ok(VcResponse::new(
        "remove_module",
        vec![("module", &module.to_string())],
    ))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;

    use abstract_core::version_control::*;

    use crate::contract;
    use speculoos::prelude::*;

    use super::*;
    use abstract_testing::prelude::{TEST_ACCOUNT_FACTORY, TEST_ADMIN, TEST_VERSION};

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_OTHER: &str = "test-other";
    const TEST_MODULE: &str = "namespace:test";

    const TEST_PROXY_ADDR: &str = "proxy";
    const TEST_MANAGER_ADDR: &str = "manager";

    /// Initialize the version_control with admin as creator and factory
    fn mock_init(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    /// Initialize the version_control with admin and updated account_factory
    fn mock_init_with_factory(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})?;
        execute_as_owner(
            deps,
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_owner(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn test_only_owner(msg: ExecuteMsg) -> VersionControlTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let _info = mock_info("not_owner", &[]);

        let res = execute_as(deps.as_mut(), "not_owner", msg);
        assert_that(&res)
            .is_err()
            .is_equal_to(VCError::Ownership(OwnershipError::NotOwner {}));

        Ok(())
    }

    use cw_controllers::AdminError;
    use cw_ownable::OwnershipError;

    mod set_admin_and_factory {
        use super::*;

        #[test]
        fn only_admin_admin() -> VersionControlTestResult {
            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: "new_admin".to_string(),
                expiry: None,
            });

            test_only_owner(msg)
        }

        #[test]
        fn only_admin_factory() -> VersionControlTestResult {
            let msg = ExecuteMsg::SetFactory {
                new_factory: "new_factory".to_string(),
            };
            test_only_owner(msg)
        }

        #[test]
        fn updates_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_admin = "new_admin";
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let transfer_res = execute_as_owner(deps.as_mut(), transfer_msg).unwrap();
            assert_eq!(0, transfer_res.messages.len());

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(deps.as_mut(), new_admin, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(Addr::unchecked(new_admin));

            Ok(())
        }

        #[test]
        fn updates_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_factory = "new_factory";
            let msg = ExecuteMsg::SetFactory {
                new_factory: new_factory.to_string(),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that!(&res).is_ok();

            let actual_factory = FACTORY.get(deps.as_ref())?.unwrap();

            assert_that!(&actual_factory).is_equal_to(Addr::unchecked(new_factory));
            Ok(())
        }
    }

    mod add_modules {
        use super::*;
        use abstract_core::objects::{module::*, module_reference::ModuleReference};
        use abstract_core::AbstractError;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Version(TEST_VERSION.into())).unwrap()
        }

        // - Query latest

        #[test]
        fn add_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module = test_module();
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res).is_ok();
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn remove_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let rm_module = test_module();

            // first add module
            let msg = ExecuteMsg::AddModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OTHER, msg)?;
            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));

            // then remove
            let msg = ExecuteMsg::RemoveModule {
                module: rm_module.clone(),
            };
            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            execute_as_owner(deps.as_mut(), msg)?;

            let module = MODULE_LIBRARY.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            Ok(())
        }

        #[test]
        fn bad_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let bad_version_module = ModuleInfo::from_id(
                TEST_MODULE,
                ModuleVersion::Version("non_compliant_version".into()),
            )?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(bad_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .matches(|e| e.to_string().contains("Invalid version"));

            let latest_version_module = ModuleInfo::from_id(TEST_MODULE, ModuleVersion::Latest)?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(latest_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Abstract(AbstractError::Assert(
                    "Module version must be set to a specific version".into(),
                )));
            Ok(())
        }

        #[test]
        fn abstract_namespace() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            let abstract_contract_id = format!("{}:{}", ABSTRACT_NAMESPACE, "test-module");
            mock_init(deps.as_mut())?;
            let new_module = ModuleInfo::from_id(&abstract_contract_id, TEST_VERSION.into())?;
            let msg = ExecuteMsg::AddModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // execute as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            execute_as_owner(deps.as_mut(), msg)?;
            let module = MODULE_LIBRARY.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn validates_module_info() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let bad_modules = vec![
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: "".to_string(),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: "".to_string(),
                },
                ModuleInfo {
                    name: "".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: "test".to_string(),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("aoeu".to_string()),
                    namespace: "".to_string(),
                },
            ];

            for bad_module in bad_modules {
                let msg = ExecuteMsg::AddModules {
                    modules: vec![(bad_module.clone(), ModuleReference::App(0))],
                };
                let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
                assert_that!(&res)
                    .named(&format!("ModuleInfo validation failed for {bad_module}"))
                    .is_err()
                    .is_equal_to(&VCError::Abstract(AbstractError::FormattingError {
                        object: "module name".into(),
                        expected: "with content".into(),
                        actual: "empty".into(),
                    }));
            }

            Ok(())
        }
    }

    mod register_os {
        use super::*;

        #[test]
        fn add_os() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init_with_factory(deps.as_mut())?;

            let test_core: AccountBase = AccountBase {
                manager: Addr::unchecked(TEST_MANAGER_ADDR),
                proxy: Addr::unchecked(TEST_PROXY_ADDR),
            };
            let msg = ExecuteMsg::AddAccount {
                account_id: 0,
                account_base: test_core.clone(),
            };

            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as admin
            let res = execute_as_owner(deps.as_mut(), msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as factory
            execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg)?;

            let account = ACCOUNT_ADDRESSES.load(&deps.storage, 0)?;
            assert_that!(&account).is_equal_to(&test_core);
            Ok(())
        }
    }

    mod configure {

        use super::*;

        #[test]
        fn update_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: TEST_OTHER.to_string(),
                expiry: None,
            });

            // as other
            let transfer_res = execute_as(deps.as_mut(), TEST_OTHER, transfer_msg.clone());
            assert_that!(&transfer_res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            execute_as_owner(deps.as_mut(), transfer_msg)?;

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(deps.as_mut(), TEST_OTHER, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(Addr::unchecked(TEST_OTHER));
            Ok(())
        }

        #[test]
        fn set_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.into(),
            };

            test_only_owner(msg.clone())?;

            execute_as_owner(deps.as_mut(), msg)?;
            let new_factory = FACTORY.query_admin(deps.as_ref())?.admin;
            assert_that!(new_factory).is_equal_to(&Some(TEST_ACCOUNT_FACTORY.into()));
            Ok(())
        }
    }
}

pub fn set_factory(deps: DepsMut, info: MessageInfo, new_admin: String) -> VCResult {
    assert_owner(deps.storage, &info.sender)?;

    let new_factory_addr = deps.api.addr_validate(&new_admin)?;
    FACTORY.set(deps, Some(new_factory_addr))?;
    Ok(Response::new().add_attribute("set_factory", new_admin))
}
