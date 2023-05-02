use abstract_core::ibc_client::*;
use cw_orch::{Contract, CwEnv};

pub use abstract_core::ibc_client::{
    ExecuteMsgFns as IbcClientExecFns, QueryMsgFns as IbcClientQueryFns,
};
use cw_orch::contract;

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
#[cfg_attr(feature = "daemon", daemon_source("abstract_ibc_client"))]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> IbcClient<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}
