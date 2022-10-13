use cosmwasm_std::StdError;

use cw_utils::ParseReplyError;
use simple_ica::SimpleIcaError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HostError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("This host does not implement any custom queries")]
    NoCustomQueries,

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    SimpleIca(#[from] SimpleIcaError),

    
    #[error("Cannot register over an existing channel")]
    ChannelAlreadyRegistered,

    #[error("Invalid reply id")]
    InvalidReplyId,
}
