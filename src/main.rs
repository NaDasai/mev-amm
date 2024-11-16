extern crate alloc;

mod constant;
mod helper;

use alloc::vec;
use alloc::vec::Vec;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::StorageAddress;
use helper::OracleReader;

/// Entry point contract
#[storage]
#[entrypoint]
pub struct Contract {
    owner: StorageAddress,
}

#[public]
impl Contract {
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        let owner_address = Address::parse_checksummed("put here", None)
            .expect("Invalid owner address");
        self.owner.set(owner_address);
        Ok(())
    }

    pub fn get_price(&mut self, oracle_name: &str) -> Result<u256, Vec<u8>> {
        let oracle_reader = OracleReader;
        oracle_reader.fetch_price(oracle_name)
    }

    pub fn get_price_with_age(&mut self, oracle_name: &str) -> Result<(u256, u256), Vec<u8>> {
        let oracle_reader = OracleReader;
        oracle_reader.fetch_price_with_age(oracle_name)
    }
}
