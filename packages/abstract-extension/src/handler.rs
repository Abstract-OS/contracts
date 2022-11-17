use abstract_sdk::base::{AbstractContract, Handler};
use cosmwasm_std::Empty;

use crate::{ExtensionContract, ExtensionError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        ExecMsg,
        InitMsg,
        QueryMsg,
        Receive,
    > Handler for ExtensionContract<Error, ExecMsg, InitMsg, QueryMsg, Receive>
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
