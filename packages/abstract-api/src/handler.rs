use abstract_sdk::{AbstractContract, Handler};
use cosmwasm_std::Empty;

use crate::{ApiContract, ApiError};

impl<Error: From<cosmwasm_std::StdError> + From<ApiError>, ExecMsg, InitMsg, QueryMsg, Receive>
    Handler for ApiContract<Error, ExecMsg, InitMsg, QueryMsg, Receive>
{
    fn contract(
        &self,
    ) -> &AbstractContract<Self, Error, ExecMsg, InitMsg, QueryMsg, Empty, Receive> {
        &self.contract
    }

    type Error = Error;

    type CustomExecMsg = ExecMsg;

    type CustomInitMsg = InitMsg;

    type CustomQueryMsg = QueryMsg;

    type CustomMigrateMsg = Empty;

    type ReceiveMsg = Receive;
}
