use boot_core::{prelude::boot_contract, BootEnvironment, BootError, Contract};
use cosmwasm_std::Empty;

use abstract_os::{
    api::{self, InstantiateMsg},
    dex::*,
    objects::{AnsAsset, AssetEntry},
    EXCHANGE, MANAGER,
};

use crate::Manager;
use boot_core::interface::ContractInstance;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct DexApi<Chain>;

impl<Chain: BootEnvironment> DexApi<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("dex"), // .with_mock(Box::new(
                                                              //     ContractWrapper::new_with_empty(
                                                              //         ::contract::execute,
                                                              //         ::contract::instantiate,
                                                              //         ::contract::query,
                                                              //     ),
                                                              // ))
        )
    }

    pub fn swap(
        &self,
        offer_asset: (&str, u128),
        ask_asset: &str,
        dex: String,
    ) -> Result<(), BootError> {
        let manager = Manager::new(MANAGER, &self.get_chain());
        let asset = AssetEntry::new(offer_asset.0);
        let ask_asset = AssetEntry::new(ask_asset);
        manager.execute_on_module(
            EXCHANGE,
            api::ExecuteMsg::<_>::App(api::ApiRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: None,
                        belief_price: None,
                    },
                },
            }),
        )?;
        Ok(())
    }
}
