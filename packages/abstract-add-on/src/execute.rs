use abstract_os::add_on::{BaseExecuteMsg, ExecuteMsg};

use abstract_sdk::{ExecuteEndpoint, Handler, IbcCallbackEndpoint};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{state::AddOnContract, AddOnError, AddOnResult};

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError> + 'static,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for AddOnContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
        // request_handler: impl FnOnce(DepsMut, Env, MessageInfo, Self, T) -> Result<Response, E>,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::App(request) => self.execute_handler()?(deps, env, info, self, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported AddOn execute message variant").into()),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AddOnContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn base_execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AddOnResult {
        match message {
            BaseExecuteMsg::UpdateConfig { ans_host_address } => {
                self.update_config(deps, info, ans_host_address)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        ans_host_address: Option<String>,
    ) -> AddOnResult {
        // self._update_config(deps, info, ans_host_address)?;
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(ans_host_address) = ans_host_address {
            state.ans_host.address = deps.api.addr_validate(ans_host_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default().add_attribute("action", "updated_ans_host_address"))
    }
}
