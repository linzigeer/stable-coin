use crate::{get_collateral_in_usd, round_to_n_decimals};
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

pub fn calc_health_factor_when_liquidate(
    price_update: &PriceUpdateV2,
    collateral_total_amount: u64,
    stablecoin_total_minted: u64,
    liquidation_threshold: u64,
) -> Option<f64> {
    let collateral_in_usd =
        get_collateral_in_usd(price_update).expect("invoke get_collateral_in_usd encounter error");
    let collateral_total_value =
        collateral_in_usd.checked_mul(collateral_total_amount.checked_div(LAMPORTS_PER_SOL)?)?;
    let health_factor = collateral_total_value as f64 * (liquidation_threshold as f64 / 100.0)
        / stablecoin_total_minted as f64;

    let rounded = round_to_n_decimals(health_factor, 4);
    Some(rounded)
}

pub fn calc_liquidatable_collateral(
    price_update: &PriceUpdateV2,
    amount_to_burn: u64,
) -> Option<u64> {
    let collateral_in_usd =
        get_collateral_in_usd(price_update).expect("invoke get_collateral_in_usd encounter error");
    let liquidatable_amount = amount_to_burn.checked_div(collateral_in_usd)?.checked_mul(LAMPORTS_PER_SOL)?;    
    
    Some(liquidatable_amount)
}
