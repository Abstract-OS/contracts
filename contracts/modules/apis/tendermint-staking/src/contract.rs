use abstract_api::{export_endpoints, ApiContract};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use abstract_os::tendermint_staking::RequestMsg;
use abstract_sdk::Execution;

use crate::error::TendermintStakeError;
use crate::staking::*;

use abstract_os::TENDERMINT_STAKING;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type TendermintStakeApi = ApiContract<TendermintStakeError, RequestMsg>;
pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

const STAKING_API: TendermintStakeApi =
    TendermintStakeApi::new(TENDERMINT_STAKING, CONTRACT_VERSION).with_execute(handle_request);

// Export handlers
#[cfg(not(feature = "library"))]
export_endpoints!(STAKING_API, TendermintStakeApi);

pub fn handle_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: TendermintStakeApi,
    msg: RequestMsg,
) -> TendermintStakeResult {
    let executor = api.executor(deps.as_ref());
    let msg = match msg {
        RequestMsg::Delegate { validator, amount } => {
            executor.execute(vec![delegate_to(&deps.querier, &validator, amount.u128())?])
        }
        RequestMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, api.target()?, &validator)?,
            };
            executor.execute(vec![undelegate_msg])
        }
        RequestMsg::UndelegateAll {} => {
            executor.execute(undelegate_all(&deps.querier, api.target()?)?)
        }

        RequestMsg::Redelegate {
            source_validator,
            destination_validator,
            amount,
        } => {
            let redelegate_msg = match amount {
                Some(amount) => redelegate(
                    &deps.querier,
                    &source_validator,
                    &destination_validator,
                    amount.u128(),
                )?,
                None => redelegate_all(
                    &deps.querier,
                    &source_validator,
                    &destination_validator,
                    api.target()?,
                )?,
            };
            executor.execute(vec![redelegate_msg])
        }
        RequestMsg::SetWithdrawAddress {
            new_withdraw_address,
        } => executor.execute(vec![update_withdraw_address(
            deps.api,
            &new_withdraw_address,
        )?]),
        RequestMsg::WithdrawDelegatorReward { validator } => {
            executor.execute(vec![withdraw_rewards(&validator)])
        }
        RequestMsg::WithdrawAllRewards {} => {
            executor.execute(withdraw_all_rewards(&deps.querier, api.target()?)?)
        }
    }?;
    Ok(Response::new().add_message(msg))
}
