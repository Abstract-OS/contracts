use abstract_boot::*;
use abstract_core::{module_factory, objects::module::ModuleInfo};
use abstract_testing::prelude::{TEST_ADMIN, TEST_VERSION};
use boot_core::{instantiate_default_mock_env, ContractInstance, Deploy};
use cosmwasm_std::Addr;
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(TEST_ADMIN);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;

    let factory = deployment.module_factory;
    let factory_config = factory.config()?;
    let expected = module_factory::ConfigResponse {
        owner: sender.into_string(),
        ans_host_address: deployment.ans_host.address()?.into(),
        version_control_address: deployment.version_control.address()?.into_string(),
    };

    assert_that!(&factory_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn caller_must_be_manager() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(TEST_ADMIN);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;

    let factory = &deployment.module_factory;
    let test_module = ModuleInfo::from_id(
        "publisher:test",
        abstract_core::objects::module::ModuleVersion::Latest,
    )?;

    let res = factory.install_module(test_module, None).unwrap_err();
    assert_that(&res.root().to_string())
        .contains("ensure that the contract is a Manager or Proxy contract");

    Ok(())
}
