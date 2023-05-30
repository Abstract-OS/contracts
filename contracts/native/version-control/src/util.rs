
use crate::{error::VCError, contract::VCResult};

use cw_asset::{Asset, AssetInfo};
use cosmwasm_std::{ Addr, StdError, CosmosMsg, MessageInfo, BankMsg, Env};
pub fn validate_sent_funds(fee: Asset, env: Env, msg_info: MessageInfo, receiver: Option<Addr>) -> VCResult<Vec<CosmosMsg>>{

    let mut fee_messages = vec![];
    match &fee.info {
        AssetInfo::Native(d) => {
            if msg_info.funds.len() != 1
                || msg_info.funds[0].denom != d.clone()
                || fee.amount != msg_info.funds[0].amount
            {
                return Err(VCError::InvalidFeePayment {
                    expected: fee,
                    sent: msg_info.funds,
                });
            }
            fee_messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.unwrap_or(env.contract.address).to_string(),
                amount: msg_info.funds,
            }))
        }
        AssetInfo::Cw20(_) => {
        	if let Some(receiver) = receiver{
        		fee_messages
            		.push(fee.transfer_from_msg(msg_info.sender, receiver)?)
            }
        },
        _ => return Err(VCError::Std(StdError::generic_err("Unreachable"))),
    }

    Ok(fee_messages)
}