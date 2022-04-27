use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use pandora_os::pandora_dapp::msg::DappExecuteMsg;
use pandora_os::pandora_dapp::msg::DappQueryMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(DappExecuteMsg),
    // Add dapp-specific messages here
    DepositStable { deposit_amount: Uint128 },
    RedeemStable { withdraw_amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(DappQueryMsg),
    // Add dapp-specific queries here
}
