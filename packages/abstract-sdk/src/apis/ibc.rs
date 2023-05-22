//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use cosmwasm_std::Addr;
use serde::Serialize;

use cosmwasm_std::to_binary;
use crate::{features::AccountIdentification, AbstractSdkResult, ModuleRegistryInterface};
use abstract_core::{
    ibc_client::{CallbackInfo, ExecuteMsg as IbcClientMsg},
    ibc_host::HostAction,
    objects::{chain_name::ChainName, module::ModuleInfo},
    proxy::ExecuteMsg, IBC_CLIENT,
    manager
};
use cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Deps};

/// Interact with other chains over IBC.
pub trait IbcInterface: AccountIdentification + ModuleRegistryInterface {
    fn ibc_client<'a>(&'a self, deps: Deps<'a>) -> IbcClient<Self> {
        IbcClient { base: self, deps }
    }
}

impl<T> IbcInterface for T where T: AccountIdentification + ModuleRegistryInterface {}

#[derive(Clone)]
pub struct IbcClient<'a, T: IbcInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: IbcInterface> IbcClient<'a, T> {
    /// Registers the ibc client to be able to use IBC capabilities
    pub fn register_ibc_client(
        &self,
    ) -> AbstractSdkResult<CosmosMsg> {
        // We start by querying the contract addressof the register ibc client module
        let ibc_client_address = self.base.module_registry(self.deps).query_module(ModuleInfo::from_id(IBC_CLIENT, abstract_core::objects::module::ModuleVersion::Latest)?)?;
        
        Ok(wasm_execute(
            self.base.manager_address(self.deps)?,
            &manager::ExecuteMsg::UpdateModuleAddresses { 
                to_add: Some(vec![(IBC_CLIENT.to_string(), ibc_client_address.to_string())]), 
                to_remove: None
            },
            vec![],
        )?
        .into())
    }
    /// A simple hepler to create and register a distant account
    pub fn create_distant_account(
        &self,
        host_chain: ChainName // The chain on which you want to create an account
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction { msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register{
                host_chain,
            }] } ,
            vec![],
        )?
        .into())
    }

    /// A simple hepler to create and register a distant account
    pub fn install_distant_app<M: Serialize>(
        &self,
        host_chain: ChainName, // The chain on which you want to create an account,
        remote_ans_host_address: Addr,
        module: ModuleInfo,
        init_msg: &M
    ) -> AbstractSdkResult<CosmosMsg> {

        self.host_action(host_chain, HostAction::Dispatch { 
            manager_msg: abstract_core::manager::ExecuteMsg::InstallModule{
                module,
                init_msg: Some(to_binary(&abstract_core::app::InstantiateMsg { 
                    base: abstract_core::app::BaseInstantiateMsg{
                        ans_host_address: remote_ans_host_address.to_string(),
                    }, 
                    module: init_msg
                }).unwrap())
            }
        }, None, 2)
    }
    /// A simple hepler install a distant api Module providing only the 
    pub fn install_distant_api<M: Serialize>(
        &self,
        host_chain: ChainName, // The chain on which you want to create an account,
        remote_ans_host_address: Addr,
        remote_version_control_address: Addr,
        module: ModuleInfo,
        init_msg: &M
    ) -> AbstractSdkResult<CosmosMsg> {

        self.host_action(host_chain, HostAction::Dispatch { 
            manager_msg: abstract_core::manager::ExecuteMsg::InstallModule{
                module,
                init_msg: Some(to_binary(&abstract_core::api::InstantiateMsg { 
                    base: abstract_core::api::BaseInstantiateMsg{
                        ans_host_address: remote_ans_host_address.to_string(),
                        version_control_address: remote_version_control_address.to_string()
                    }, 
                    module: init_msg
                }).unwrap())
            }
        }, None, 2)
    }

    /// A simple hepler to execute on a module
    pub fn execute_on_module<M: Serialize>(
        &self,
        host_chain: ChainName, // The chain on which you want to create an account,
        module_id: String,
        exec_msg: &M
    ) -> AbstractSdkResult<CosmosMsg> {

        self.host_action(host_chain, HostAction::Dispatch { 
            manager_msg: abstract_core::manager::ExecuteMsg::ExecOnModule { 
                module_id,
                exec_msg: to_binary(exec_msg)?,
            }
        }, None, 2)
    }

    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: ChainName,
        action: HostAction,
        callback: Option<CallbackInfo>,
        retries: u8,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain,
                    action,
                    callback_info: callback,
                    retries,
                }],
            },
            vec![],
        )?
        .into())
    }
    /// IbcClient the provided coins from the Account to its proxy on the `receiving_chain`.
    pub fn ics20_transfer(
        &self,
        receiving_chain: ChainName,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: receiving_chain,
                    funds,
                }],
            },
            vec![],
        )?
        .into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;
    const TEST_HOST_CHAIN: &str = "host_chain";

    /// Tests that a host_action can be built with no callback
    #[test]
    fn test_host_action_no_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());
        let expected_retries = 0;
        let msg = client.host_action(
            TEST_HOST_CHAIN.into(),
            HostAction::Balances {},
            None,
            expected_retries,
        );
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Balances {},
                    callback_info: None,
                    retries: expected_retries,
                }],
            })
            .unwrap(),
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to(expected);
    }

    /// Tests that a host_action can be built with a callback with more retries
    #[test]
    fn test_host_action_with_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_callback = CallbackInfo {
            id: "callback_id".to_string(),
            receiver: "callback_receiver".to_string(),
        };

        let expected_retries = 50;
        let actual = client.host_action(
            TEST_HOST_CHAIN.into(),
            HostAction::Balances {},
            Some(expected_callback.clone()),
            expected_retries,
        );

        assert_that!(actual).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Balances {},
                    callback_info: Some(expected_callback),
                    retries: expected_retries,
                }],
            })
            .unwrap(),
            funds: vec![],
        });

        assert_that!(actual.unwrap()).is_equal_to(expected);
    }

    /// Tests that the ics_20 transfer can be built and that the funds are passed into the sendFunds message not the execute message
    #[test]
    fn test_ics20_transfer() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_funds = coins(100, "denom");

        let msg = client.ics20_transfer(TEST_HOST_CHAIN.into(), expected_funds.clone());
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: TEST_HOST_CHAIN.into(),
                    funds: expected_funds,
                }],
            })
            .unwrap(),
            // ensure empty
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to(expected);
    }
}
