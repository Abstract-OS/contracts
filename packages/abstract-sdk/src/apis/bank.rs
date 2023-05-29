//! # Bank
//! The Bank object handles asset transfers to and from the Account.

use crate::features::AccountIdentification;
use crate::AccountAction;
use crate::{ans_resolve::Resolve, features::AbstractNameService, AbstractSdkResult};
use core::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Deps, Env};
use cw_asset::Asset;

/// Query and Transfer assets from and to the Abstract Account.
pub trait TransferInterface: AbstractNameService + AccountIdentification {
    /**
        API for transferring funds to and from the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let bank: Bank<MockModule>  = module.bank(deps.as_ref());
        ```
    */
    fn bank<'a>(&'a self, deps: Deps<'a>) -> Bank<Self> {
        Bank { base: self, deps }
    }
}

impl<T> TransferInterface for T where T: AbstractNameService + AccountIdentification {}

/**
    API for transferring funds to and from the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let bank: Bank<MockModule>  = module.bank(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Bank<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: TransferInterface> Bank<'a, T> {
    /// Get the balances of the provided assets.
    pub fn balances(&self, assets: &[AssetEntry]) -> AbstractSdkResult<Vec<Asset>> {
        assets
            .iter()
            .map(|asset| self.balance(asset))
            .collect::<AbstractSdkResult<Vec<Asset>>>()
    }
    /// Get the balance of the provided asset.
    pub fn balance(&self, asset: &AssetEntry) -> AbstractSdkResult<Asset> {
        let resolved_info = asset.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        let balance =
            resolved_info.query_balance(&self.deps.querier, self.base.proxy_address(self.deps)?)?;
        Ok(Asset::new(resolved_info, balance))
    }

    /// Transfer the provided funds from the Account to the recipient.
    /// ```
    /// # use cosmwasm_std::{Addr, Response, Deps, DepsMut, MessageInfo};
    /// # use abstract_core::objects::AnsAsset;
    /// # use abstract_core::objects::ans_host::AnsHost;
    /// # use abstract_sdk::{
    /// #    features::{AccountIdentification, AbstractNameService, ModuleIdentification},
    /// #    TransferInterface, AbstractSdkResult, Execution,
    /// # };
    /// # struct MockModule;
    /// # impl AccountIdentification for MockModule {
    /// #    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
    /// #       unimplemented!("Not needed for this example")
    /// #   }
    /// # }
    /// #
    /// # impl ModuleIdentification for MockModule {
    /// #   fn module_id(&self) -> &'static str {
    /// #      "mock_module"
    /// #  }
    /// # }
    /// #
    /// # impl AbstractNameService for MockModule {
    /// #   fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
    /// #     unimplemented!("Not needed for this example")
    /// #  }
    /// # }
    /// fn transfer_asset_to_sender(app: MockModule, deps: DepsMut, info: MessageInfo, requested_asset: AnsAsset) -> AbstractSdkResult<Response> {
    ///     let bank = app.bank(deps.as_ref());
    ///     let executor = app.executor(deps.as_ref());    
    ///     let transfer_action = bank.transfer(vec![requested_asset.clone()], &info.sender)?;
    ///
    ///     let transfer_msg = executor.execute(vec![transfer_action])?;
    ///
    ///     Ok(Response::new()
    ///         .add_message(transfer_msg)
    ///         .add_attribute("recipient", info.sender)
    ///         .add_attribute("asset_sent", requested_asset.to_string()))
    /// }
    /// ```
    pub fn transfer<R: Transferable>(
        &self,
        funds: Vec<R>,
        recipient: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
            .map(Into::into)
    }

    /// Move funds from the contract into the Account.
    pub fn deposit<R: Transferable>(&self, funds: Vec<R>) -> AbstractSdkResult<AccountAction> {
        let recipient = self.base.proxy_address(self.deps)?;
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
            .map(Into::into)
    }

    /// Withdraw funds from the Account to this contract.
    pub fn withdraw<R: Transferable>(
        &self,
        env: &Env,
        funds: Vec<R>,
    ) -> AbstractSdkResult<AccountAction> {
        let recipient = &env.contract.address;
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<CosmosMsg>, _>>()
            .map_err(Into::into)
            .map(Into::into)
    }

    /// Deposit coins into the Account
    pub fn deposit_coins(&self, coins: Vec<Coin>) -> AbstractSdkResult<CosmosMsg> {
        let recipient = self.base.proxy_address(self.deps)?.into_string();
        Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient,
            amount: coins,
        }))
    }
}

/// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
pub trait Transferable {
    /// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset>;
}

impl Transferable for &AnsAsset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps)?)
    }
}

impl Transferable for AnsAsset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps)?)
    }
}

impl Transferable for Asset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        Ok(self)
    }
}

impl Transferable for Coin {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        Ok(Asset::from(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod transfer_coins {
        use super::*;

        #[test]
        fn transfer_asset_to_sender() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let expected_recipient = Addr::unchecked("recipient");

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.transfer(coins.clone(), &expected_recipient);

            assert_that!(actual_res).is_ok();

            let expected_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: expected_recipient.to_string(),
                amount: coins,
            });

            assert_that!(actual_res.unwrap().messages()[0]).is_equal_to(&expected_msg);
        }
    }

    // transfer must be tested via integration test

    mod deposit_coins {
        use super::*;

        #[test]
        fn deposit_coins() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.deposit_coins(coins.clone());

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_PROXY.to_string(),
                amount: coins,
            });

            assert_that!(actual_res)
                .is_ok()
                .is_equal_to::<CosmosMsg>(expected_msg);
        }
    }

    mod withdraw_coins {
        use super::*;

        #[test]
        fn withdraw_coins() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let env = mock_env();

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.withdraw(&env, coins.clone());

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: env.contract.address.to_string(),
                amount: coins,
            });

            assert_that!(actual_res.unwrap().messages()[0]).is_equal_to::<CosmosMsg>(expected_msg);
        }
    }
}
