use crate::{
    dex_trait::{Fee, FeeOnInput, Return, Spread},
    error::DexError,
    DEX,
};

use cosmwasm_std::{Addr, Coin, CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgExitPool, MsgJoinPool, MsgSwapExactAmountIn, MsgSwapExactAmountOut, SwapAmountInRoute,
};

pub const OSMOSIS: &str = "osmosis";
// Source https://github.com/wasmswap/wasmswap-contracts
pub struct Osmosis {}

/// Osmosis app-chain dex implementation
impl DEX for Osmosis {
    fn over_ibc(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        OSMOSIS
    }

    fn swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let token_out_denom = match ask_asset {
            AssetInfo::Native { .. } => "uosmo".to_string(),
            AssetInfo::Cw20(contract_addr) => contract_addr.to_string(),
            _ => return Err(DexError::Cw1155Unsupported),
        };

        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom,
        }];

        let token_in = Coin::try_from(offer_asset)?;

        let swap_msg: CosmosMsg = MsgSwapExactAmountIn {
            sender,
            routes,
            token_in: Some(token_in.into()),
            token_out_min_amount: Uint128::zero().to_string(),
        }
        .into();

        return Ok(vec![swap_msg]);
    }

    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        // The offer_assets have already been sent to the host contract
        // The ask_assets are the assets we want to receive
        // Generate the swap message(s) between the offer and ask assets
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn provide_liquidity(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
    }

    fn provide_liquidity_symmetric(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _lp_token: Asset,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn simulate_swap(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }
}
