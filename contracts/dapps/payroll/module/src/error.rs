use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;
use pandora::treasury::dapp_base::error::BaseDAppError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PayrollError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not the base token")]
    WrongToken {},

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The actual amount of tokens transfered is different from the claimed amount.")]
    InvalidAmount {},
}
