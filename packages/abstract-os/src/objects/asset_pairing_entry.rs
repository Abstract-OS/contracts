use std::fmt::Display;

pub type DexName = String;

/// The key for an asset pairing
pub type DexAssetPairing = (String, String, DexName);
