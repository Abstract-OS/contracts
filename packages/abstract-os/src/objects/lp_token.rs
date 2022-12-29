use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::constants::{ASSET_DELIMITER, TYPE_DELIMITER};

use crate::dex::DexName;
use crate::objects::{AssetEntry, PoolMetadata};
use cosmwasm_std::StdError;

/// A key for the token that represents Liquidity Pool shares on a dex
/// @todo: move into dex package
#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Default,
)]
pub struct LpToken {
    pub dex: DexName,
    pub assets: Vec<String>,
}

impl LpToken {
    pub fn new<T: ToString>(dex_name: T, assets: &[String]) -> Self {
        Self {
            dex: dex_name.to_string(),
            assets: assets.to_vec(),
        }
    }

    /// Return a vector of the assets in the pool as [`AssetEntry`]s
    pub fn assets(&self) -> Vec<AssetEntry> {
        self.assets.iter().map(AssetEntry::from).collect()
    }
}

/// Try from an asset entry that should be formatted as "dex_name/asset1,asset2"
impl TryFrom<AssetEntry> for LpToken {
    type Error = StdError;

    fn try_from(asset: AssetEntry) -> Result<Self, Self::Error> {
        let segments = asset.as_str().split(TYPE_DELIMITER).collect::<Vec<_>>();

        if segments.len() != 2 {
            return Err(StdError::generic_err(format!(
                "Invalid asset entry: {}",
                asset
            )));
        }

        // get the dex name, like "junoswap"
        let dex_name = segments[0].to_string();

        // get the assets, like "crab,junox" and split them
        let assets: Vec<String> = segments[1]
            .split(ASSET_DELIMITER)
            .map(|s| s.to_string())
            .collect();

        if assets.len() < 2 {
            return Err(StdError::generic_err(format!(
                "Must be at least 2 assets in an LP token: {}",
                asset
            )));
        }

        Ok(Self {
            dex: dex_name,
            assets,
        })
    }
}

/// Transform into a string formatted as "dex_name/asset1,asset2"
impl Display for LpToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let assets = self.assets.join(ASSET_DELIMITER);

        write!(f, "{}{}{}", self.dex, TYPE_DELIMITER, assets)
    }
}

impl From<LpToken> for AssetEntry {
    fn from(lp_token: LpToken) -> Self {
        AssetEntry::from(lp_token.to_string())
    }
}

/// Build the LP token from pool metadata.
impl From<PoolMetadata> for LpToken {
    fn from(pool: PoolMetadata) -> Self {
        Self {
            dex: pool.dex,
            assets: pool.assets,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    mod implementation {
        use super::*;

        #[test]
        fn new_works() {
            let dex_name = "junoswap";
            let assets = vec!["crab".to_string(), "junox".to_string()];
            let actual = LpToken::new(dex_name, assets.as_slice());

            let expected = LpToken {
                dex: dex_name.to_string(),
                assets,
            };
            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn assets_returns_asset_entries() {
            let dex_name = "junoswap";
            let assets = vec!["crab".to_string(), "junox".to_string()];
            let lp_token = LpToken::new(dex_name, assets.as_slice());
            let expected = vec![AssetEntry::from("crab"), AssetEntry::from("junox")];

            assert_that!(lp_token.assets()).is_equal_to(expected);
        }
    }

    mod from_asset_entry {
        use super::*;

        #[test]
        fn test_from_asset_entry() {
            let lp_token = LpToken::try_from(AssetEntry::new("junoswap/crab,junox")).unwrap();
            assert_that!(lp_token.dex).is_equal_to("junoswap".to_string());
            assert_that!(lp_token.assets)
                .is_equal_to(vec!["crab".to_string(), "junox".to_string()]);
        }

        #[test]
        fn test_from_invalid_asset_entry() {
            let lp_token = LpToken::try_from(AssetEntry::new("junoswap/"));
            assert_that!(&lp_token).is_err();
        }

        #[test]
        fn test_fewer_than_two_assets() {
            let lp_token = LpToken::try_from(AssetEntry::new("junoswap/crab"));
            assert_that!(&lp_token).is_err();
        }
    }

    mod into_asset_entry {
        use super::*;

        #[test]
        fn into_asset_entry_works() {
            let lp_token = LpToken::new("junoswap", &["crab".to_string(), "junox".to_string()]);
            let expected = AssetEntry::new("junoswap/crab,junox");

            assert_that!(lp_token.into()).is_equal_to(expected);
        }
    }

    mod from_pool_metadata {
        use super::*;
        use crate::objects::PoolType;

        #[test]
        fn test_from_pool_metadata() {
            let assets = vec!["crab".to_string(), "junox".to_string()];
            let dex = "junoswap".to_string();

            let pool = PoolMetadata {
                dex: dex.clone(),
                pool_type: PoolType::Stable,
                assets: assets.clone(),
            };
            let lp_token = LpToken::from(pool);
            assert_that!(lp_token.dex).is_equal_to(dex);
            assert_that!(lp_token.assets).is_equal_to(assets);
        }
    }
}
