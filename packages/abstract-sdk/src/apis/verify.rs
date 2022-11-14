//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an OS, OS admin, etc.
use abstract_os::{
    manager::state::OS_ID,
    version_control::{state::OS_ADDRESSES, Core},
};
use cosmwasm_std::{Addr, QuerierWrapper, StdError};

use cosmwasm_std::StdResult;

use crate::features::{Identification, Versioning};

pub trait Verification: Identification + Versioning {
    fn verify(&self) -> Verify<Self> {
        Verify { base: self }
    }
}

impl<T> Verification for T where T: Identification + Versioning {}

/// Endpoint for OS address verification
pub struct Verify<'a, T: Verification> {
    base: &'a T,
}

impl<'a, T: Verification> Verify<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> StdResult<Core> {
        let os_id = OS_ID.query(&self.base.querier(), maybe_manager.clone())?;
        let maybe_os =
            OS_ADDRESSES.query(&self.base.querier(), self.base.version_registry()?, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {} is not active.",
                os_id
            ))),
            Some(core) => {
                if &core.manager != maybe_manager {
                    Err(StdError::generic_err(
                    "Proposed manager is not the manager of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> StdResult<Core> {
        let os_id = OS_ID.query(&self.base.querier(), maybe_proxy.clone())?;
        let maybe_os =
            OS_ADDRESSES.query(&self.base.querier(), self.base.version_registry()?, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {} is not active.",
                os_id
            ))),
            Some(core) => {
                if &core.proxy != maybe_proxy {
                    Err(StdError::generic_err(
                        "Proposed proxy is not the proxy of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }
}

