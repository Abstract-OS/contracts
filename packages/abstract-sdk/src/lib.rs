#![doc(html_logo_url = "https://raw.githubusercontent.com/Abstract-OS/assets/mainline/logo.svg")]
#![doc = include_str!("../README.md")]
#![doc(test(attr(
    warn(unused),
    deny(warnings),
    // W/o this, we seem to get some bogus warning about `extern crate zbus`.
    // allow(unused_extern_crates),
)))]

//! ### Abstract Base
//!
//! To use an API either construct a [`feature object`](crate::feature_objects) or use an Abstract base contract as the starting-point of your application.  
//! The available base contracts are:
//!
//! |  Kind          | Migratable | Installable  |
//! |----------------|---------------|---------------|
//! | [App](https://crates.io/crates/abstract-app) | ✅  | ✅ |
//! | [API](https://crates.io/crates/abstract-api)   | ❌ | ✅ |
//! | [IBC-host](https://crates.io/crates/abstract-ibc-host)   | ✅ | ❌ |
//!
//! Each base supports a set of endpoints that can accept custom handlers. These handlers can be added to the base using a static builder pattern.
//! All the available endpoints are discussed [here](crate::base::endpoints).
//!
//!
//! ## Usage
//!
//! Add `abstract-sdk` to your `Cargo.toml` by running:
//! ```bash
//! $ cargo add abstract-sdk
//! ```
//!   # use abstract_sdk::{feature_objects::VersionControlContract, base::features::{Identification, AbstractNameService, ModuleIdentification}};
//!   # use cosmwasm_std::{StdResult, Deps, MessageInfo, CosmosMsg, Addr};
//!   # use abstract_sdk::feature_objects::AnsHost;
//!   # use abstract_sdk::AbstractSdkResult;
//!
//!   # pub struct MyContract {
//!   #     
//!   # }
//!   #
//!   # impl Identification for MyContract {
//!   #     fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
//!   #         Ok(Addr::unchecked("just_an_example"))
//!   #     }
//!   # }
//!   # impl ModuleIdentification for MyContract {
//!   #     fn module_id(&self) -> &'static str { "my_contract" }
//!   # }
//!
//!   # impl AbstractNameService for MyContract {
//!   #     fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
//!   #         Ok(AnsHost{address: Addr::unchecked("just_an_example")})
//!   #     }
//!   # }
//!   use abstract_sdk::TransferInterface;
//!
//!   fn forward_deposit(deps: Deps, my_contract: MyContract, message_info: MessageInfo) -> StdResult<CosmosMsg> {
//!       let send_deposit_to_vault_msg = my_contract.bank(deps).deposit_coins(message_info.funds)?;
//!       Ok(send_deposit_to_vault_msg)
//!   }
//!   #
//!   # fn main() {}
//!   ```

pub type AbstractSdkResult<T> = Result<T, crate::error::AbstractSdkError>;

pub extern crate abstract_macros as macros;
pub extern crate abstract_os as os;

mod ans_resolve;
mod apis;

pub mod base;
pub mod cw_helpers;
mod error;
pub mod feature_objects;

pub use error::{AbstractSdkError, EndpointError};

pub use crate::apis::{
    bank::TransferInterface, dex::DexInterface, execution::Execution, ibc::IbcInterface,
    modules::ModuleInterface, respond::AbstractResponse, vault::VaultInterface,
    verify::OsVerification, version_registry::ModuleRegistryInterface,
};

pub mod features {
    //! # Feature traits
    //! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.  
    //!
    //! You can easily create and provide your own API for other smart-contract developers by using these features as trait bounds.
    pub use crate::base::features::*;
}

pub use ans_resolve::Resolve;

/// Common state-store namespaces.
pub mod namespaces {
    pub use abstract_os::objects::common_namespace::*;
}

/// Abstract reserved version control entries.
pub mod register {
    pub use abstract_os::registry::*;
}
