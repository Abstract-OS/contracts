use abstract_boot::{
    Abstract, AnsHost, Manager, ModuleFactory, OSFactory, Proxy, VersionControl, OS,
};
use abstract_os::{ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY, VERSION_CONTROL};
use boot_core::{prelude::ContractInstance, Mock};
use cw_multi_test::ContractWrapper;

pub const ROOT_USER: &str = "root_user";

pub fn init_abstract_env(chain: Mock) -> anyhow::Result<(Abstract<Mock>, OS<Mock>)> {
    let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let mut os_factory = OSFactory::new(OS_FACTORY, chain.clone());
    let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let mut manager = Manager::new(MANAGER, chain.clone());
    let mut proxy = Proxy::new(PROXY, chain.clone());

    ans_host
        .as_instance_mut()
        .set_mock(Box::new(ContractWrapper::new_with_empty(
            ::ans_host::contract::execute,
            ::ans_host::contract::instantiate,
            ::ans_host::contract::query,
        )));

    os_factory.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::abstract_os_factory::contract::execute,
            ::abstract_os_factory::contract::instantiate,
            ::abstract_os_factory::contract::query,
        )
        .with_reply_empty(::abstract_os_factory::contract::reply),
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

    let deployment = Abstract {
        chain,
        version: "1.0.0".parse()?,
        ans_host,
        os_factory,
        version_control,
        module_factory,
    };

    let os_core = OS { manager, proxy };

    Ok((deployment, os_core))
}
