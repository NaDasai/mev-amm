// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{ alloy_primitives::{ Address, U256 }, msg, prelude::* };

use crate::erc20::{ Erc20, Erc20Params, Erc20Error };

/// Immutable definitions
struct LpTokenParams;
impl Erc20Params for LpTokenParams {
    const NAME: &'static str = "LpToken";
    const SYMBOL: &'static str = "Lp";
    const DECIMALS: u8 = 18;
}

// Define some persistent storage using the Solidity ABI.
// `AMM` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct AMM {
        Erc20<LpTokenParams> erc20;
    }
}

/// Declare that `Amm` is a contract with the following external methods.
#[public]
impl AMM {
    /// Mints tokens
    pub fn mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(msg::sender(), value)?;
        Ok(())
    }

    /// Mints tokens to another address
    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(to, value)?;
        Ok(())
    }

    /// Burns tokens
    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), value)?;
        Ok(())
    }
}
