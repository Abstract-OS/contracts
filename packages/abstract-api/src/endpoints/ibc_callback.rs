use crate::{ApiContract, ApiError};
use abstract_sdk::base::endpoints::IbcCallbackEndpoint;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > IbcCallbackEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
}
