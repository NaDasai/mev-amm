// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{ alloy_primitives::{ Address, U256 }, msg, contract, call::RawCall, prelude::* };
use crate::erc20::{ Erc20, Erc20Params, Erc20Error };
use alloy_primitives::Uint;

sol_interface! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint);
    }
}

/// Immutable definitions
struct LpTokenParams;
impl Erc20Params for LpTokenParams {
    const NAME: &'static str = "LpToken";
    const SYMBOL: &'static str = "Lp";
    const DECIMALS: u8 = 18;
}

const MINIMUM_LIQUIDITY: u64 = 1_000;

// Define some persistent storage using the Solidity ABI.
// `AMM` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct AMM {
        // Pair
        address factory;
        address token0;
        address token1;
        uint112 reserve0;
        uint112 reserve1;
        // LP
        Erc20<LpTokenParams> lptoken;
    }
}

/// Declare that `Amm` is a contract with the following external methods.
#[public]
impl AMM {
    pub fn initialize(&mut self, token0: Address, token1: Address) -> Result<(), Vec<u8>> {
        if self.factory.get() != Address::ZERO {
            return Err("Already initialized".into());
        }
        self.factory.set(msg::sender());
        self.token0.set(token0);
        self.token1.set(token1);
        Ok(())
    }

    pub fn add_liquidity(&mut self, to: Address) -> Result<U256, Vec<u8>> {
        let (_reserve0, _reserve1) = self.get_reserves();
        let balance0 = IERC20::new(self.token0.get()).balance_of(&*self, contract::address())?;
        let balance1 = IERC20::new(self.token1.get()).balance_of(&*self, contract::address())?;
        let amount0 = balance0.checked_sub(_reserve0).ok_or("balance0-reserve0 overflow")?;
        let amount1 = balance1.checked_sub(_reserve1).ok_or("balance1-reserve1 overflow")?;

        let total_supply = self.lptoken.total_supply();

        let liquidity = if total_supply == U256::ZERO {
            self.lptoken.mint(Address::ZERO, U256::from(MINIMUM_LIQUIDITY))?;
            self
                .sqrt(amount0.checked_mul(amount1).unwrap())
                .checked_sub(U256::from(MINIMUM_LIQUIDITY))
                .ok_or("sqrt underflow")?
        } else {
            self.min(
                amount0.checked_mul(total_supply).unwrap().checked_div(_reserve0).unwrap(),
                amount1.checked_mul(total_supply).unwrap().checked_div(_reserve1).unwrap()
            )
        };

        if liquidity == U256::ZERO {
            return Err("Liquidity is zero".into());
        }

        Ok(liquidity)
    }

    pub fn remove_liquidity(&mut self, to: Address) -> Result<(U256, U256), Vec<u8>> {
        let (_reserve0, _reserve1) = self.get_reserves();
        let _token0 = self.token0.get();
        let _token1 = self.token1.get();
        let mut balance0 = IERC20::new(_token0).balance_of(&*self, contract::address())?;
        let mut balance1 = IERC20::new(_token1).balance_of(&*self, contract::address())?;
        let liquidity = self.lptoken.balance_of(contract::address());

        let total_supply = self.lptoken.total_supply();
        let amount0 = liquidity.checked_mul(balance0).unwrap() / total_supply;
        let amount1 = liquidity.checked_mul(balance1).unwrap() / total_supply;
        if amount0 == U256::ZERO || amount1 == U256::ZERO {
            return Err("INSUFFICIENT_LIQUIDITY_BURNED".into());
        }
        self.lptoken.burn(contract::address(), liquidity)?;
        self.safe_transfer(_token0, to, amount0)?;
        self.safe_transfer(_token1, to, amount1)?;
        balance0 = IERC20::new(_token0).balance_of(&*self, contract::address())?;
        balance1 = IERC20::new(_token1).balance_of(&*self, contract::address())?;

        self.update(balance0, balance1, _reserve0, _reserve1);

        Ok((amount0, amount1))
    }

    pub fn swap(
        &mut self,
        amount0_out: U256,
        amount1_out: U256,
        to: Address,
        data: Vec<u8>
    ) -> Result<(), Vec<u8>> {
        if amount0_out == U256::ZERO || amount1_out == U256::ZERO {
            return Err("INSUFFICIENT_OUTPUT_AMOUNT".into());
        }
        let (_reserve0, _reserve1) = self.get_reserves();
        if amount0_out >= _reserve0 || amount1_out >= _reserve1 {
            return Err("INSUFFICIENT_LIQUIDITY".into());
        }

        let token0 = IERC20::new(self.token0.get());
        let token1 = IERC20::new(self.token1.get());
        if amount0_out > U256::ZERO {
            self.safe_transfer(self.token0.get(), to, amount0_out)?;
        }
        if amount1_out > U256::ZERO {
            self.safe_transfer(self.token1.get(), to, amount1_out)?;
        }
        if !data.is_empty() {
            RawCall::new().call(to, &data)?;
        }
        let balance0 = token0.balance_of(&*self, contract::address())?;
        let balance1 = token1.balance_of(&*self, contract::address())?;
        let amount0_in = balance0.saturating_sub(_reserve0.saturating_sub(amount0_out));
        let amount1_in = balance1.saturating_sub(_reserve1.saturating_sub(amount1_out));
        if amount0_in == U256::ZERO && amount1_in == U256::ZERO {
            return Err("INSUFFICIENT_INPUT_AMOUNT".into());
        }
        let balance0_adjusted = balance0
            .checked_mul(U256::from(1000))
            .unwrap()
            .checked_sub(amount0_in.checked_mul(U256::from(3)).unwrap())
            .ok_or("balance0Adjusted underflow")?;
        let balance1_adjusted = balance1
            .checked_mul(U256::from(1000))
            .unwrap()
            .checked_sub(amount1_in.checked_mul(U256::from(3)).unwrap())
            .ok_or("balance1Adjusted underflow")?;
        let k = _reserve0.checked_mul(_reserve1).unwrap().checked_mul(U256::from(1000)).unwrap();
        if balance0_adjusted.checked_mul(balance1_adjusted).unwrap() < k {
            return Err("K".into());
        }
        self.update(balance0, balance1, _reserve0, _reserve1);

        Ok(())
    }

    /// Mints tokens
    pub fn mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.lptoken.mint(msg::sender(), value)?;
        Ok(())
    }

    /// Mints tokens to another address
    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.lptoken.mint(to, value)?;
        Ok(())
    }

    /// Burns tokens
    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.lptoken.burn(msg::sender(), value)?;
        Ok(())
    }

    pub fn update(&mut self, balance0: U256, balance1: U256, reserve0: U256, reserve1: U256) {
        if reserve0 > U256::ZERO && reserve1 > U256::ZERO {
            self.reserve0.set(Uint::<112, 2>::from(balance0));
            self.reserve1.set(Uint::<112, 2>::from(balance1));
        }
    }

    pub fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256
    ) -> Result<(), Vec<u8>> {
        let calldata: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
        let ret = { RawCall::new().call(token, &calldata) };

        let success = match ret {
            Ok(_) => true,
            Err(_) => false,
        };
        let data = ret.unwrap_or_default();
        let is_true_bool = data.len() == 32 && data[31] == 1 && data[..31].iter().all(|&x| x == 0);
        if !(success && (data.len() == 0 || is_true_bool)) {
            return Err("MEV AMM: TRANSFER_FAILED".into());
        }

        Ok(())
    }

    pub fn get_reserves(&self) -> (U256, U256) {
        (U256::from(self.reserve0.get()), U256::from(self.reserve1.get()))
    }

    fn min(&self, x: U256, y: U256) -> U256 {
        if x < y { x } else { y }
    }

    // babylonian method (https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Babylonian_method)
    fn sqrt(&self, y: U256) -> U256 {
        if y > U256::from(3) {
            let mut z = y;
            let mut x = y / U256::from(2) + U256::from(1);
            while x < z {
                z = x;
                x = (y / x + x) / U256::from(2);
            }
            z
        } else if y != U256::ZERO {
            U256::from(1)
        } else {
            U256::ZERO
        }
    }
}
