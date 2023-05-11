use abstract_core::ibc_client::*;
use cw_orch::prelude::*;

pub use abstract_core::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use cw_orch::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
