use abstract_sdk::os::api::*;

use abstract_sdk::os::base;
use abstract_sdk::os::tendermint_staking::*;
use cosmwasm_std::Empty;

use crate::AbstractOS;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};

pub type TMintStakingApi<Chain> = AbstractOS<
    Chain,
    ExecuteMsg<RequestMsg>,
    base::InstantiateMsg<BaseInstantiateMsg>,
    abstract_sdk::os::api::QueryMsg<abstract_sdk::os::tendermint_staking::QueryMsg>,
    Empty,
>;

impl<Chain: TxHandler + Clone> TMintStakingApi<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("tendermint_staking"), // .with_mock(Box::new(
                                                                             //     ContractWrapper::new_with_empty(
                                                                             //         ::contract::execute,
                                                                             //         ::contract::instantiate,
                                                                             //         ::contract::query,
                                                                             //     ),
                                                                             // ))
        )
    }
}
