
pub fn calcSpotPrice(tokenBalanceIn: U256, tokenWeightIn: U256, tokenBalanceOut: U256, 
    tokenWeightOut: U256, swapFee: U256) -> Result<(), Error> {
    let numer = tokenBalanceIn / tokenWeightIn;
    let denom = tokenBalanceOut / tokenWeightOut;
    let ratio = numer / denom;
    let scale = 1 / (1 - swapFee);
    let spotPrice = ratio * scale ;
    return spotPrice 
    }

pub fn calcOutGivenIn(tokenBalanceIn: U256, tokenWeightIn: U256, tokenBalanceOut: U256, 
    tokenWeightOut: U256, tokenAmountIn: U256, swapFee: U256) -> Result<(), Error> { 
        let weightRatio = tokenWeightIn / tokenWeightOut;
        let adjustedIn = (1 - swapFee) * tokenAmountIn;
        let y  =  (tokenBalanceIn / (tokenBalanceIn + adjustedIn));
        let foo =  y.powf(weightRatio);
        let bar = 1 - foo;
        tokenBalanceOut*bar; 
    }
pub fn calcInGivenOut(tokenBalanceIn: U256, tokenWeightIn: U256, tokenBalanceOut: U256, 
    tokenWeightOut: U256, tokenAmountOut: U256, swapFee: U256) -> Result<(), Error> {
        let weightRatio = tokenWeightOut / tokenWeightIn;
        let y = tokenBalanceOut / (tokenBalanceOut - tokenAmountOut);
        let foo = y.powf(weightRatio)-1;
        tokenBalanceIn * (foo) / (1-swapFee)

    }

println!("{}",calcSpotPrice(100, 100, 100,500));

