/*
The following code copy logic from CowHelper.sol
https://www.codeslaw.app/contracts/ethereum/0x703bd8115e6f21a37bb5df97f78614ca72ad7624
*/ 


#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use stylus_sdk::prelude::*;
use stylus_sdk::alloy_primitives::{Address, U256};

use crate::math::calc_out_given_in;
#[storage]
#[entrypoint]
pub struct Contract {
    factory: Address,       // Factory address
    app_data: [u8; 32],     // App data, equivalent to _APP_DATA
}

#[public]
impl Contract {
    /// Initialize the contract with the factory address
    pub fn init(&mut self, factory: Address) -> Result<(), Vec<u8>> {
        self.factory = factory;
        self.app_data = IBCoWFactory::new(factory).app_data(); // Fetch the app data from the factory
        Ok(())
    }

    /// Retrieve tokens from a pool
    pub fn tokens(&self, pool: Address) -> Result<Vec<Address>, Vec<u8>> {
        // Verify if the pool is deployed by the factory
        if !IBCoWFactory::new(self.factory).is_b_pool(pool) {
            return Err(b"PoolDoesNotExist".to_vec());
        }

        let pool_contract = IBCoWPool::new(pool);
        let tokens = pool_contract.get_final_tokens()?;

        // Validate pool conditions
        if tokens.len() != 2 {
            return Err(b"PoolDoesNotExist".to_vec());
        }
        if pool_contract.get_normalized_weight(tokens[0])? != pool_contract.get_normalized_weight(tokens[1])? {
            return Err(b"PoolDoesNotExist".to_vec());
        }

        Ok(tokens)
    }

    /// Create a order based on pool and prices
    pub fn order(&self, pool: Address, prices: Vec<U256>) -> Result<(Order, Vec<Interaction>, Vec<Interaction>, Vec<u8>), Vec<u8>> {
        let tokens = self.tokens(pool)?;

        // Prepare order parameters
        let params = GetTradeableOrderParams {
            pool,
            token0: tokens[0],
            token1: tokens[1],
            price_numerator: prices[1],
            price_denominator: prices[0],
            app_data: self.app_data,
        };

        let mut order = get_tradeable_order(params)?;

        // Fetch balances and calculate sell amount
        let balance_token0 = IERC20::new(tokens[0]).balance_of(pool)?;
        let balance_token1 = IERC20::new(tokens[1]).balance_of(pool)?;

        let (balance_in, balance_out) = if order.buy_token == tokens[0] {
            (balance_token0, balance_token1)
        } else {
            (balance_token1, balance_token0)
        };

        // Use the imported calc_out_given_in function
        order.sell_amount = calc_out_given_in(balance_in, balance_out, order.buy_amount)?;

        // Generate signature
        let domain_separator = IBCoWPool::new(pool).solution_settler_domain_separator()?;
        let order_commitment = order.hash(domain_separator)?;

        let pre_interactions = vec![Interaction {
            target: pool,
            value: U256::zero(),
            call_data: IBCoWPool::commit(order_commitment)?,
        }];

        // NOTE: In Stylus, empty vectors must be explicitly defined
        let post_interactions: Vec<Interaction> = vec![];

        Ok((order, pre_interactions, post_interactions, encode_signature(pool, &order)))
    }
}

fn encode_signature(pool: Address, order: &Order) -> Vec<u8> {
    let eip1271sig = abi_encode(order);
    abi_encode_packed(pool, eip1271sig)
}