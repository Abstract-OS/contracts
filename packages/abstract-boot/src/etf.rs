use abstract_sdk::os::{app::MigrateMsg, etf::*};
use boot_core::prelude::*;
use boot_core::{Contract, IndexResponse, TxResponse};

#[boot_contract(EtfInstantiateMsg, EtfExecuteMsg, EtfQueryMsg, MigrateMsg)]
pub struct ETF;

impl<Chain: BootEnvironment> ETF<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("etf"), // .with_mock(Box::new(
                                                              //     ContractWrapper::new_with_empty(
                                                              //         ::contract::execute,
                                                              //         ::contract::instantiate,
                                                              //         ::contract::query,
                                                              //     ),
                                                              // ))
        )
    }
}
