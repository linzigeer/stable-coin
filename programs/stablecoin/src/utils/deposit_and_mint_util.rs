use crate::constants::{MAX_AGE, SOL_USD_FEED_ID};
use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_2022::{mint_to, MintTo};
use num_traits::cast::FromPrimitive;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

pub fn get_collateral_in_usd(price_update: &Account<PriceUpdateV2>) -> Result<u64> {
    let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
    let price = price_update.get_price_no_older_than(&Clock::get()?, MAX_AGE, &feed_id)?;

    Ok(price.price as u64)
}

pub fn calc_mintable_amount(
    deposit_collateral_amount: u64,
    collateral_in_usd: u64,
    max_ltv: u64,
) -> Option<u64> {
    let collateral_value =
        collateral_in_usd.checked_mul(deposit_collateral_amount / LAMPORTS_PER_SOL)?;
    if collateral_value == 0 {
        return None;
    }

    let value = collateral_value as f64 * max_ltv as f64 / 100.0;

    u64::from_f64(value)
}

pub fn calc_redeemable_amount(
    amount_to_burn:u64,
    collateral_in_usd: u64
) -> Option<u64> {
    amount_to_burn.checked_div(collateral_in_usd)?.checked_mul(LAMPORTS_PER_SOL)
}

pub fn calc_health_factor_when_deposit_collateral_and_mint_new_tokens(
    price_update: &Account<PriceUpdateV2>,
    amount_deposited: u64,
    amount_minted: u64,
    amount_to_deposit: u64,
    amount_to_mint: u64,
    liquidation_threshold: u64,
) -> Option<f64> {
    let collateral_in_usd =
        get_collateral_in_usd(price_update).expect("invoke get_collateral_in_usd encounter error");
    let collateral_total_value = collateral_in_usd.checked_mul(
        amount_deposited
            .checked_add(amount_to_deposit)?
            .checked_div(LAMPORTS_PER_SOL)?,
    )?;
    let health_factor = collateral_total_value as f64 * (liquidation_threshold as f64 / 100.0)
        / (amount_minted as f64 + amount_to_mint as f64);

    let rounded = round_to_n_decimals(health_factor, 4);
    Some(rounded)
}

pub fn calc_health_factor_when_burn_tokens_and_redeem_collateral(
    price_update: &Account<PriceUpdateV2>,
    amount_deposited: u64,
    amount_minted: u64,
    amount_to_redeem: u64,
    amount_to_burn: u64,
    liquidation_threshold: u64,
) -> Option<f64> {
    let collateral_in_usd =
        get_collateral_in_usd(price_update).expect("invoke get_collateral_in_usd encounter error");
    let collateral_total_value = collateral_in_usd.checked_mul(
        amount_deposited
            .checked_sub(amount_to_redeem)?
            .checked_div(LAMPORTS_PER_SOL)?,
    )?;
    let health_factor = collateral_total_value as f64 * (liquidation_threshold as f64 / 100.0)
        / (amount_minted as f64 - amount_to_burn as f64);

    let rounded = round_to_n_decimals(health_factor, 4);
    Some(rounded)
}

pub fn check_health_factor_when_deposit_collateral_and_mint_new_tokens(
    price_update: &Account<PriceUpdateV2>,
    amount_deposited: u64,
    amount_minted: u64,
    amount_to_deposit: u64,
    amount_to_mint: u64,
    liquidation_threshold: u64,
) -> Result<()> {
    let health_factor = calc_health_factor_when_deposit_collateral_and_mint_new_tokens(
        price_update,
        amount_deposited,
        amount_minted,
        amount_to_deposit,
        amount_to_mint,
        liquidation_threshold,
    )
    .expect("invoke method check_health_factor encounter error!");
    if health_factor < 1.0 {
        return Err(ErrorCode::HealthFactorLessThanOne.into());
    }

    Ok(())
}

pub fn check_health_factor_when_burn_tokens_and_redeem_collateral(
    price_update: &Account<PriceUpdateV2>,
    amount_deposited: u64,
    amount_minted: u64,
    amount_to_redeem: u64,
    amount_to_burn: u64,
    liquidation_threshold: u64,
) -> Result<()> {
    let health_factor = calc_health_factor_when_burn_tokens_and_redeem_collateral(
        price_update,
        amount_deposited,
        amount_minted,
        amount_to_redeem,
        amount_to_burn,
        liquidation_threshold,
    )
        .expect("invoke method check_health_factor encounter error!");
    if health_factor < 1.0 {
        return Err(ErrorCode::HealthFactorLessThanOne.into());
    }

    Ok(())
}

pub fn round_to_n_decimals(original_num: f64, n_decimals: i32) -> f64 {
    let base = 10.0f64.powi(n_decimals);
    (original_num * base).round() / base
}

pub fn deposit_collateral<'info>(
    amount_to_deposit: u64,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    program: AccountInfo<'info>,
) -> Result<()> {
    let accounts = Transfer { from, to };
    let cpi_ctx = CpiContext::new(program, accounts);
    transfer(cpi_ctx, amount_to_deposit)
}

pub fn mint_stable_coins<'info>(
    amount_to_mint: u64,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    program: AccountInfo<'info>,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    let accounts = MintTo {
        mint,
        to,
        authority,
    };
    let cpi_ctx = CpiContext::new_with_signer(program, accounts, seeds);
    mint_to(cpi_ctx, amount_to_mint)
}
