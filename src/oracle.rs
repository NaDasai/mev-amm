use stylus_sdk::prelude::*;
use stylus_sdk::alloy_primitives::Address;
use crate::constant::get_oracle;

/// OracleReader encapsulates logic for interacting with oracles
#[public]
impl OracleReader {
    /// Reads the price from a given oracle contract
    pub fn fetch_price(&mut self, oracle_name: &str) -> Result<u256, Vec<u8>> {
        let contract_address = get_oracle(oracle_name)?;

        // Interact with the oracle contract
        self.call_view(contract_address)
    }

    /// Reads the price and age from a given oracle contract
    pub fn fetch_price_with_age(&mut self, oracle_name: &str) -> Result<(u256, u256), Vec<u8>> {
        // Retrieve the oracle address
        let contract_address = get_oracle(oracle_name)?;

        // Interact with the oracle contract
        self.call_view_with_age(contract_address)
    }

    /// Low-level call to fetch price from an oracle contract
    pub fn call_view(&mut self, contract_address: Address) -> Result<u256, Vec<u8>> {
        let external_contract = IChronicle::new(contract_address);
        let config = Call::new_in(self);
        let price: u256 = external_contract.read(config)?;
        Ok(price)
    }

    /// Low-level call to fetch price and age from an oracle contract
    pub fn call_view_with_age(&mut self, contract_address: Address) -> Result<(u256, u256), Vec<u8>> {
        let external_contract = IChronicle::new(contract_address);
        let config = Call::new_in(self);
        let (price, age): (u256, u256) = external_contract.readWithAge(config)?;
        Ok((price, age))
    }
}

// 