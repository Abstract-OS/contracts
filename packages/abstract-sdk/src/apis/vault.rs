//! # Vault
//! The Vault object provides function for querying balances and asset values for the OS.

use crate::helpers::cosmwasm_std::wasm_smart_query;
use abstract_os::{
    objects::{proxy_asset::ProxyAsset, AssetEntry},
    proxy::{
        state::VAULT_ASSETS, AssetsResponse, HoldingValueResponse, QueryMsg, TotalValueResponse,
    },
};
use cosmwasm_std::{Deps, StdError, StdResult, Uint128};

use super::{AbstractNameService, Identification};

/// Retrieve asset-registration information from the OS.
/// Query asset values and balances.
pub trait VaultInterface: AbstractNameService + Identification {
    fn vault<'a>(&'a self, deps: Deps<'a>) -> Vault<Self> {
        Vault { base: self, deps }
    }
}

impl<T> VaultInterface for T where T: AbstractNameService + Identification {}

#[derive(Clone)]
pub struct Vault<'a, T: VaultInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: VaultInterface> Vault<'a, T> {
    /// Query the total value denominated in the base asset
    /// The provided address must implement the TotalValue Query
    pub fn query_total_value(&self) -> StdResult<Uint128> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response: TotalValueResponse = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::TotalValue {},
        )?)?;

        Ok(response.value)
    }

    /// RawQuery the proxy for a ProxyAsset
    pub fn asset(&self, asset: &AssetEntry) -> StdResult<ProxyAsset> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response = VAULT_ASSETS.query(&querier, proxy_address, asset.clone())?;
        response.ok_or_else(|| {
            StdError::generic_err(format!(
                "Asset {} is not registered as an asset on your proxy contract.",
                asset
            ))
        })
    }

    /// Query the holding value denominated in the base asset
    /// The provided address must implement the HoldingValue Query
    pub fn balance_value(&self, asset_entry: &AssetEntry) -> StdResult<Uint128> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response: HoldingValueResponse = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::HoldingValue {
                identifier: asset_entry.to_string(),
            },
        )?)?;

        Ok(response.value)
    }

    /// Query the token amount of a specific asset
    /// The asset must be registered in the proxy contract
    pub fn asset_value(
        &self,
        asset_entry: &AssetEntry,
        amount: Option<Uint128>,
    ) -> StdResult<Uint128> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;

        let response: TotalValueResponse = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::TokenValue {
                identifier: asset_entry.to_string(),
                amount,
            },
        )?)?;

        Ok(response.value)
    }

    /// List ProxyAssets raw
    pub fn enabled_assets_list(&self) -> StdResult<(Vec<AssetEntry>, AssetEntry)> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;

        let mut asset_keys = vec![];
        let mut base_asset: Option<AssetEntry> = None;
        let mut resp: AssetsResponse = querier.query_wasm_smart(
            &proxy_address,
            &QueryMsg::Assets {
                page_token: None,
                page_size: None,
            },
        )?;
        while !resp.assets.is_empty() {
            let page_token = resp.assets.last().unwrap().0.clone();
            for (k, v) in resp.assets {
                maybe_set_base(&v, &mut base_asset);
                asset_keys.push(k);
            }
            resp = querier.query_wasm_smart(
                &proxy_address,
                &QueryMsg::Assets {
                    page_token: Some(page_token.to_string()),
                    page_size: None,
                },
            )?;
        }
        Ok((asset_keys, base_asset.unwrap()))
    }

    /// List ProxyAssets raw
    pub fn proxy_assets_list(&self) -> StdResult<Vec<(AssetEntry, ProxyAsset)>> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;

        let mut assets = vec![];
        let mut resp: AssetsResponse = querier.query_wasm_smart(
            &proxy_address,
            &QueryMsg::Assets {
                page_token: None,
                page_size: None,
            },
        )?;
        while !resp.assets.is_empty() {
            let page_token = resp.assets.last().unwrap().0.clone();
            assets.append(resp.assets.as_mut());
            resp = querier.query_wasm_smart(
                &proxy_address,
                &QueryMsg::Assets {
                    page_token: Some(page_token.to_string()),
                    page_size: None,
                },
            )?;
        }
        Ok(assets)
    }
}

#[inline(always)]
fn maybe_set_base(value: &ProxyAsset, base: &mut Option<AssetEntry>) {
    if value.value_reference.is_none() {
        *base = Some(value.asset.clone());
    }
}
