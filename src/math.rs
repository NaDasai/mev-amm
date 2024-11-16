
pub fn calc_spot_price(
    token_balance_in: U256,
    token_weight_in: U256,
    token_balance_out: U256,
    token_weight_out: U256,
    swap_fee: U256,
) -> Result<U256, Error> {
    let numer = token_balance_in.checked_div(token_weight_in).ok_or(Error::Math)?;
    let denom = token_balance_out.checked_div(token_weight_out).ok_or(Error::Math)?;
    let ratio = numer.checked_div(denom).ok_or(Error::Math)?;
    let scale = U256::from(1).checked_div(U256::from(1) - swap_fee).ok_or(Error::Math)?;
    let spot_price = ratio.checked_mul(scale).ok_or(Error::Math)?;
    Ok(spot_price)
}

pub fn calc_out_given_in(
    token_balance_in: U256,
    token_weight_in: U256,
    token_balance_out: U256,
    token_weight_out: U256,
    token_amount_in: U256,
    swap_fee: U256,
) -> Result<U256, Error> {
    let weight_ratio = token_weight_in.checked_div(token_weight_out).ok_or(Error::Math)?;
    let adjusted_in = (U256::from(1) - swap_fee)
        .checked_mul(token_amount_in)
        .ok_or(Error::Math)?;
    let y = token_balance_in
        .checked_div(token_balance_in + adjusted_in)
        .ok_or(Error::Math)?;
    let foo = y.checked_pow(weight_ratio).ok_or(Error::Math)?;
    let bar = U256::from(1) - foo;
    let result = token_balance_out.checked_mul(bar).ok_or(Error::Math)?;
    Ok(result)
}

pub fn calc_in_given_out(
    token_balance_in: U256,
    token_weight_in: U256,
    token_balance_out: U256,
    token_weight_out: U256,
    token_amount_out: U256,
    swap_fee: U256,
) -> Result<U256, Error> {
    let weight_ratio = token_weight_out.checked_div(token_weight_in).ok_or(Error::Math)?;
    let y = token_balance_out
        .checked_div(token_balance_out - token_amount_out)
        .ok_or(Error::Math)?;
    let foo = y.checked_pow(weight_ratio).ok_or(Error::Math)?;
    let fee_adjusted = (foo - U256::from(1))
        .checked_div(U256::from(1) - swap_fee)
        .ok_or(Error::Math)?;
    let result = token_balance_in.checked_mul(fee_adjusted).ok_or(Error::Math)?;
    Ok(result)
}

