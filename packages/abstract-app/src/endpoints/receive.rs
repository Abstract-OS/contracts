use crate::ReceiveEndpoint;
use crate::{error::AppError, state::AppContract};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::SdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ReceiveEndpoint
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
