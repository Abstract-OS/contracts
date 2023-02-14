use crate::{error::AbstractOsError, AbstractResult};
use cosmwasm_std::{Addr, Deps};

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum ModuleReference {
    /// Core Abstract Contracts
    Core(u64),
    /// Native Abstract Contracts
    Native(Addr),
    /// Installable apis
    Api(Addr),
    /// Installable apps
    App(u64),
    /// A stand-alone contract
    Standalone(u64),
}

impl ModuleReference {
    /// Validates that addresses are valid
    pub fn validate(&self, deps: Deps) -> AbstractResult<()> {
        match self {
            ModuleReference::Native(addr) => {
                deps.api.addr_validate(addr.as_str())?;
            }
            ModuleReference::Api(addr) => {
                deps.api.addr_validate(addr.as_str())?;
            }
            _ => (),
        };
        Ok(())
    }

    pub fn unwrap_core(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::Core(v) => Ok(*v),
            _ => Err(AbstractOsError::Assert(
                "module reference not a core module.".to_string(),
            )),
        }
    }

    pub fn unwrap_native(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Native(addr) => Ok(addr.clone()),
            _ => Err(AbstractOsError::Assert(
                "module reference not a native module.".to_string(),
            )),
        }
    }

    pub fn unwrap_api(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Api(addr) => Ok(addr.clone()),
            _ => Err(AbstractOsError::Assert(
                "module reference not an api module.".to_string(),
            )),
        }
    }

    pub fn unwrap_app(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::App(v) => Ok(*v),
            _ => Err(AbstractOsError::Assert(
                "module reference not an app module.".to_string(),
            )),
        }
    }

    pub fn unwrap_standalone(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::Standalone(v) => Ok(*v),
            _ => Err(AbstractOsError::Assert(
                "module reference not a standalone module.".to_string(),
            )),
        }
    }

    /// Unwraps the module reference and returns the address of the module.
    /// Throws an error if the module reference is not an address.
    pub fn unwrap_addr(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Native(addr) => Ok(addr.clone()),
            ModuleReference::Api(addr) => Ok(addr.clone()),
            _ => Err(AbstractOsError::Assert(
                "module reference not a native or api module.".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    #[test]
    fn core() {
        let core = ModuleReference::Core(1);
        assert_eq!(core.unwrap_core().unwrap(), 1);
        assert!(core.unwrap_native().is_err());
        assert!(core.unwrap_api().is_err());
        assert!(core.unwrap_app().is_err());
        assert!(core.unwrap_standalone().is_err());
    }
    #[test]
    fn native() {
        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert!(native.unwrap_core().is_err());
        assert_eq!(native.unwrap_native().unwrap(), Addr::unchecked("addr"));
        assert!(native.unwrap_api().is_err());
        assert!(native.unwrap_app().is_err());
        assert!(native.unwrap_standalone().is_err());
    }

    #[test]
    fn api() {
        let api = ModuleReference::Api(Addr::unchecked("addr"));
        assert!(api.unwrap_core().is_err());
        assert!(api.unwrap_native().is_err());
        assert_eq!(api.unwrap_api().unwrap(), Addr::unchecked("addr"));
        assert!(api.unwrap_app().is_err());
        assert!(api.unwrap_standalone().is_err());
    }

    #[test]
    fn app() {
        let app = ModuleReference::App(1);
        assert!(app.unwrap_core().is_err());
        assert!(app.unwrap_native().is_err());
        assert!(app.unwrap_api().is_err());
        assert_eq!(app.unwrap_app().unwrap(), 1);
        assert!(app.unwrap_standalone().is_err());
    }

    #[test]
    fn standalone() {
        let standalone = ModuleReference::Standalone(1);
        assert!(standalone.unwrap_core().is_err());
        assert!(standalone.unwrap_native().is_err());
        assert!(standalone.unwrap_api().is_err());
        assert!(standalone.unwrap_app().is_err());
        assert_eq!(standalone.unwrap_standalone().unwrap(), 1);
    }

    #[test]
    fn unwrap_addr() {
        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert_eq!(native.unwrap_addr().unwrap(), Addr::unchecked("addr"));
        let api = ModuleReference::Api(Addr::unchecked("addr"));
        assert_eq!(api.unwrap_addr().unwrap(), Addr::unchecked("addr"));

        let core = ModuleReference::Core(1);
        assert!(core.unwrap_addr().is_err());
    }

    #[test]
    fn test_validate_happy_path() {
        let deps = mock_dependencies();

        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert_that!(native.validate(deps.as_ref())).is_ok();

        let api = ModuleReference::Api(Addr::unchecked("addr"));
        assert_that!(api.validate(deps.as_ref())).is_ok();

        let core = ModuleReference::Core(1);
        assert_that!(core.validate(deps.as_ref())).is_ok();

        let app = ModuleReference::App(1);
        assert_that!(app.validate(deps.as_ref())).is_ok();

        let standalone = ModuleReference::Standalone(1);
        assert_that!(standalone.validate(deps.as_ref())).is_ok();
    }

    #[test]
    fn test_validate_bad_address() {
        let deps = mock_dependencies();

        let native = ModuleReference::Native(Addr::unchecked(""));
        assert_that!(native.validate(deps.as_ref())).is_err();

        let api = ModuleReference::Api(Addr::unchecked(""));
        assert_that!(api.validate(deps.as_ref())).is_err();
    }
}
