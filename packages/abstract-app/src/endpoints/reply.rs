use crate::ReplyEndpoint;
use crate::{AppContract, AppError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ReplyEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
}
