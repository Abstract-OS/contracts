use crate::{
    contract::{DexApi, DexResult},
    error::DexError,
    DEX,
};

use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Deps, Fraction, QueryRequest,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use osmosis_std::types::osmosis::gamm::v1beta1::MsgSwapExactAmountIn;
pub const OSMOSIS: &str = "osmosis";
// Source https://github.com/wasmswap/wasmswap-contracts
pub struct Osmosis {}

impl DEX for Osmosis {
    fn name(&self) -> &'static str {
        OSMOSIS
    }
    fn swap(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> DexResult {
        todo!()
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> DexResult {
        todo!()
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        api: DexApi,
        pair_address: Addr,
        offer_asset: Asset,
        other_assets: Vec<AssetInfo>,
    ) -> DexResult {
        todo!()
    }

    fn withdraw_liquidity(
        &self,
        deps: Deps,
        api: &DexApi,
        pair_address: Addr,
        lp_token: Asset,
    ) -> DexResult {
        todo!()
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Uint128, Uint128, Uint128, bool), DexError> {
        todo!()
    }
}

// fn denom_and_asset_match(denom: &Denom, asset: &AssetInfo) -> Result<bool, DexError> {
//     match denom {
//         Denom::Native(denom_name) => match asset {
//             cw_asset::AssetInfoBase::Native(asset_name) => Ok(denom_name == asset_name),
//             cw_asset::AssetInfoBase::Cw20(_asset_addr) => Ok(false),
//             cw_asset::AssetInfoBase::Cw1155(_, _) => Err(DexError::Cw1155Unsupported),
//             _ => todo!(),
//         },
//         Denom::Cw20(denom_addr) => match asset {
//             cw_asset::AssetInfoBase::Native(_asset_name) => Ok(false),
//             cw_asset::AssetInfoBase::Cw20(asset_addr) => Ok(denom_addr == asset_addr),
//             cw_asset::AssetInfoBase::Cw1155(_, _) => Err(DexError::Cw1155Unsupported),
//             _ => todo!(),
//         },
//     }
// }

// fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
//     let mut msgs = vec![];
//     for asset in assets {
//         if let AssetInfo::Cw20(addr) = &asset.info {
//             let msg = cw20_junoswap::Cw20ExecuteMsg::IncreaseAllowance {
//                 spender: spender.to_string(),
//                 amount: asset.amount,
//                 expires: None,
//             };
//             msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: addr.to_string(),
//                 msg: to_binary(&msg)?,
//                 funds: vec![],
//             }))
//         }
//     }
//     Ok(msgs)
// }

// fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
//     let mut coins = vec![];
//     for asset in assets {
//         if let AssetInfo::Native(denom) = &asset.info {
//             coins.push(Coin::new(asset.amount.u128(), denom.clone()));
//         }
//     }
//     coins
// }
