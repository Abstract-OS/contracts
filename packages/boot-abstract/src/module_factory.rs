use abstract_sdk::os::module_factory::*;

// use crate::extension::get_extension_init_msgs;
use crate::AbstractOS;
use boot_core::{BootError, Contract, IndexResponse, TxHandler, TxResponse};

pub type ModuleFactory<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> ModuleFactory<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("module_factory"), // .with_mock(Box::new(
                                                                         //     ContractWrapper::new_with_empty(
                                                                         //         ::contract::execute,
                                                                         //         ::contract::instantiate,
                                                                         //         ::contract::query,
                                                                         //     ),
                                                                         // ))
        )
    }

    pub fn change_ans_host_addr(&self, mem_addr: String) -> Result<TxResponse<Chain>, BootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                ans_host_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
    }

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), BootError> {
    //     let msgs = get_extension_init_msgs(mem_addr,version_control_addr);
    //     // TODO: Add version management support
    //     let binaries = msgs
    //         .iter()
    //         .map(|(name, msg)| ((name.clone(), "v0.1.0".to_string()), msg.clone()))
    //         .collect::<Vec<_>>();
    //     self.0
    //         .execute(
    //             &ExecuteMsg::UpdateFactoryBinaryMsgs {
    //                 to_add: binaries,
    //                 to_remove: vec![(LIQUIDITY_INTERFACE.to_string(), "v0.1.0".to_string())],
    //             },
    //             &vec![],
    //         )
    //         ?;
    //     Ok(())
    // }
}
