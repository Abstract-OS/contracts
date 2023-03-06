use crate::{AnsHost, IbcClient, Manager, ModuleFactory, OSFactory, Proxy, VersionControl};
use abstract_os::{
    objects::OsId, ANS_HOST, IBC_CLIENT, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY,
    VERSION_CONTROL,
};
use boot_core::{BootEnvironment, IndexResponse, StateInterface, TxHandler};

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: BootEnvironment>(
    chain: Chain,
) -> (
    AnsHost<Chain>,
    OSFactory<Chain>,
    VersionControl<Chain>,
    ModuleFactory<Chain>,
    IbcClient<Chain>,
)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let os_factory = OSFactory::new(OS_FACTORY, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let ibc_client = IbcClient::new(IBC_CLIENT, chain);
    (
        ans_host,
        os_factory,
        version_control,
        module_factory,
        ibc_client,
    )
}

pub fn get_os_core_contracts<Chain: BootEnvironment>(
    chain: Chain,
    os_id: Option<OsId>,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    if let Some(os_id) = os_id {
        let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let core = version_control.get_os_core(os_id).unwrap();
        chain.state().set_address(MANAGER, &core.manager);
        chain.state().set_address(PROXY, &core.proxy);
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    } else {
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    }
}
