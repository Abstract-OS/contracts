use abstract_core::ibc_host::*;
use boot_core::{contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

#[contract(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: BootEnvironment> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("abstract_osmosis_host");
        Self(contract)
    }
}
