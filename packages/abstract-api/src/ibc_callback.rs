use abstract_sdk::{IbcCallbackEndpoint, IbcCallbackHandlerFn};

use crate::{ApiContract, ApiError};

impl<'a, T, C, E: From<cosmwasm_std::StdError> + From<ApiError>> IbcCallbackEndpoint
    for ApiContract<'a, T, E, C>
{
    type ContractError = E;

    fn callback_handler(
        &self,
        id: &str,
    ) -> Option<IbcCallbackHandlerFn<Self, Self::ContractError>> {
        for ibc_callback_handler in self.ibc_callbacks {
            if ibc_callback_handler.0 == id {
                return Some(ibc_callback_handler.1);
            }
        }
        None
    }
}
