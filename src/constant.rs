// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use stylus_sdk::alloy_primitives::Address;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::StorageAddress;

// Oracle addresses
pub const ARB_ORACLE: &str = "0x91Fa05bCab98aD3DdEaE33DF7213EE8642e3c66c";
pub const BTC_ORACLE: &str = "0x898D1aB819a24880F636416df7D1493C94143262";
pub const ETH_ORACLE: &str = "0x898D1aB819a24880F636416df7D1493C94143262";
pub const GYD_ORACLE: &str = "0x88Ee016dadDCa8061bf6D566585dF6c8aBfED7bb";

/// Retrieve an oracle address by name
pub fn get_oracle(oracle_name: &str) -> Result<Address, Vec<u8>> {
    match oracle_name {
        "ARB" => Address::parse_checksummed(ARB_ORACLE, None),
        "BTC" => Address::parse_checksummed(BTC_ORACLE, None),
        "ETH" => Address::parse_checksummed(ETH_ORACLE, None),
        "GYD" => Address::parse_checksummed(GYD_ORACLE, None),
        _ => Err(format!("Unknown oracle: {}", oracle_name).into_bytes()),
    }
}