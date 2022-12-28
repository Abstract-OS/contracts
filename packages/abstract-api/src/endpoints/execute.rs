use crate::{error::ApiError, state::ApiContract, ApiResult};
use abstract_os::api::ApiExecuteMsg;
use abstract_sdk::{
    base::{
        endpoints::{ExecuteEndpoint, IbcCallbackEndpoint, ReceiveEndpoint},
        Handler,
    },
    os::api::{BaseExecuteMsg, ExecuteMsg},
    Execution, ModuleInterface, Verification,
};
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg: Serialize + JsonSchema + ApiExecuteMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        let sender = &info.sender;
        match msg {
            ExecuteMsg::App(request) => {
                let core = match request.proxy_address {
                    Some(addr) => {
                        let proxy_addr = deps.api.addr_validate(&addr)?;
                        let traders = self.traders.load(deps.storage, proxy_addr)?;
                        if traders.contains(sender) {
                            self.os_register(deps.as_ref())
                                .assert_proxy(&deps.api.addr_validate(&addr)?)?
                        } else {
                            self.os_register(deps.as_ref())
                                .assert_manager(sender)
                                .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?
                        }
                    }
                    None => self
                        .os_register(deps.as_ref())
                        .assert_manager(sender)
                        .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?,
                };
                self.target_os = Some(core);
                self.execute_handler()?(deps, env, info, self, request.request)
            }
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported api execute message variant").into()),
        }
    }
}

/// The api-contract base implementation.
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> ApiResult {
        match message {
            BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            BaseExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
        }
    }

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &mut self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ApiError> {
        let core = self
            .os_register(deps)
            .assert_manager(&info.sender)
            .map_err(|_| ApiError::UnauthorizedApiRequest {})?;
        self.target_os = Some(core);
        let dependencies = self.dependencies();
        let mut msgs: Vec<CosmosMsg> = vec![];
        let applications = self.modules(deps);
        for dep in dependencies {
            let api_addr = applications.module_address(dep.id);
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api_addr?.into_string(),
                msg: to_binary(&BaseExecuteMsg::UpdateTraders {
                    to_add: None,
                    to_remove: Some(vec![env.contract.address.to_string()]),
                })?,
                funds: vec![],
            }));
        }
        self.executor(deps)
            .execute_with_response(msgs, "remove api from dependencies")
            .map_err(Into::into)
    }

    /// Remove traders from the api.
    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ApiError> {
        // Either manager or proxy can add/remove traders.
        // This allows other apis to automatically add themselves, allowing for api-cross-calling.
        let core = self
            .os_register(deps.as_ref())
            .assert_manager(&info.sender)?;

        // Manager can only change traders for associated proxy
        let proxy = core.proxy;

        let mut traders = self
            .traders
            .may_load(deps.storage, proxy.clone())?
            .unwrap_or_default();

        // Handle the addition of traders
        if let Some(to_add) = to_add {
            for trader in to_add {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.insert(trader_addr) {
                    return Err(ApiError::TraderAlreadyPresent { trader });
                }
            }
        }

        // Handling the removal of traders
        if let Some(to_remove) = to_remove {
            for trader in to_remove {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.remove(&trader_addr) {
                    return Err(ApiError::TraderNotPresent { trader });
                }
            }
        }

        self.traders.save(deps.storage, proxy.clone(), &traders)?;
        Ok(Response::new().add_attribute("action", format!("update_{}_traders", proxy)))
    }
}

#[cfg(test)]
mod tests {

    use abstract_os::api::{BaseInstantiateMsg, InstantiateMsg};
    use abstract_sdk::base::InstantiateEndpoint;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Empty,
    };
    use thiserror::Error;

    use super::*;
    use abstract_testing::*;

    type TestApi = ApiContract<TestError, Empty, Empty, Empty, Empty>;
    type ApiTestResult = Result<(), TestError>;

    #[derive(Error, Debug, PartialEq)]
    enum TestError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),
    }

    #[test]
    fn add_trader() -> ApiTestResult {
        let api = TestApi::new("mock", "v1.9.9", None);
        let env = mock_env();
        let info = mock_info(TEST_MANAGER, &vec![]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::querier();
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            app: Empty {},
        };
        api.instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg)?;

        let mut api = TestApi::new("mock", "v1.9.9", None);
        let msg = BaseExecuteMsg::UpdateTraders {
            to_add: None,
            to_remove: None,
        };
        api.base_execute(deps.as_mut(), env, info, msg)?;

        Ok(())
    }
}
