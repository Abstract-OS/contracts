#![allow(unused)]
pub mod mock_modules;

pub const OWNER: &str = "owner";
pub const TEST_COIN: &str = "ucoin";

use ::abstract_manager::contract::CONTRACT_VERSION;
use abstract_api::mock::{BootMockApi, MockInitMsg};
use abstract_boot::{
    Abstract, AccountFactory, AnsHost, Manager, ModuleFactory, Proxy, VCExecFns, VersionControl,
};
use abstract_boot::{AbstractAccount, ApiDeployer};
use abstract_core::version_control::AccountBase;
use abstract_core::{objects::gov_type::GovernanceDetails, PROXY};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, VERSION_CONTROL};
use boot_core::{BootExecute, CallAs, ContractInstance, Mock};
use boot_core::{ContractWrapper, StateInterface};
use cosmwasm_std::Addr;
use semver::Version;

pub(crate) type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub(crate) fn create_default_account(
    factory: &AccountFactory<Mock>,
) -> anyhow::Result<AbstractAccount<Mock>> {
    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(OWNER).to_string(),
    })?;
    Ok(account)
}

use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_MODULE_ID};

pub(crate) fn init_mock_api(
    chain: Mock,
    deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<BootMockApi<Mock>> {
    deployment
        .version_control
        .claim_namespaces(TEST_ACCOUNT_ID, vec!["tester".to_string()]);
    let mut staking_api = BootMockApi::new(TEST_MODULE_ID, chain);
    let version: Version = version
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;
    staking_api.deploy(version, MockInitMsg)?;
    Ok(staking_api)
}
