use crate::{error::AddOnError, state::AddOnContract};

use abstract_sdk::{ReceiveEndpoint, ReceiveHandlerFn};

use serde::{de::DeserializeOwned, Serialize};

impl<
        'a,
        T,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
        R,
    > ReceiveEndpoint for AddOnContract<'a, T, E, R>
{
    type ContractError = E;
    type ReceiveMsg = R;

    fn receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::ContractError>> {
        self.receive_handler
    }
}
