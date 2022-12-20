use abstract_boot::{
    os_factory::OSFactory, AnsHost, Deployment, Manager, ModuleFactory, Proxy, VersionControl, OS,
};

use abstract_os::{ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY, VERSION_CONTROL};
use boot_core::{prelude::ContractInstance, Mock};

use cw_multi_test::ContractWrapper;

pub const PROXY_CONTRACT: &str = "proxy_addr";
pub const ROOT_USER: &str = "root_user";
pub const ANS_HOST_CONTRACT: &str = "ans_host_addr";
pub const TEST_CREATOR: &str = "creator";
pub const DEFAULT_VERSION: &str = "1.0.0";


pub fn init_test_env<'a>(chain: &'a Mock) -> anyhow::Result<(Deployment<'a, Mock>, OS<Mock>)> {
    let mut ans_host = AnsHost::new(ANS_HOST, chain);
    let mut os_factory = OSFactory::new(OS_FACTORY, chain);
    let mut version_control = VersionControl::new(VERSION_CONTROL, chain);
    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain);
    let mut manager = Manager::new(MANAGER, chain);
    let mut proxy = Proxy::new(PROXY, chain);

    ans_host
        .as_instance_mut()
        .set_mock(Box::new(ContractWrapper::new_with_empty(
            ::ans_host::contract::execute,
            ::ans_host::contract::instantiate,
            ::ans_host::contract::query,
        )));

    os_factory.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::os_factory::contract::execute,
            ::os_factory::contract::instantiate,
            ::os_factory::contract::query,
        )
        .with_reply_empty(::os_factory::contract::reply),
    ));

    module_factory.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::module_factory::contract::execute,
            ::module_factory::contract::instantiate,
            ::module_factory::contract::query,
        )
        .with_reply_empty(::module_factory::contract::reply),
    ));

    version_control.as_instance_mut().set_mock(Box::new(
        cw_multi_test::ContractWrapper::new_with_empty(
            ::version_control::contract::execute,
            ::version_control::contract::instantiate,
            ::version_control::contract::query,
        ),
    ));

    manager
        .as_instance_mut()
        .set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::manager::contract::execute,
            ::manager::contract::instantiate,
            ::manager::contract::query,
        )));

    proxy
        .as_instance_mut()
        .set_mock(Box::new(cw_multi_test::ContractWrapper::new_with_empty(
            ::proxy::contract::execute,
            ::proxy::contract::instantiate,
            ::proxy::contract::query,
        )));

    // do as above for the rest of the contracts

    let deployment = Deployment {
        chain: chain,
        version: "1.0.0".parse()?,
        ans_host,
        os_factory,
        version_control,
        module_factory,
    };

    let os_core = OS { manager, proxy };

    Ok((deployment, os_core))
}
