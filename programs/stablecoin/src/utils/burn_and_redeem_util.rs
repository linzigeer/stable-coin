use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_2022::{burn, Burn};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use crate::utils::{get_collateral_in_usd, round_to_n_decimals};

pub fn calc_health_factor_when_burn_tokens_and_redeem_collateral(
    price_update: &PriceUpdateV2,
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
pub fn check_health_factor_when_burn_tokens_and_redeem_collateral(
    price_update: &PriceUpdateV2,
    amount_deposited: u64,
    amount_minted: u64,
    amount_to_redeem: u64,
    amount_to_burn: u64,
    liquidation_threshold: u64,
    configured_min_health_factor:u64
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
    msg!("health factor after try to burn and redeem:{}", health_factor);
    if health_factor < configured_min_health_factor as f64/ 100.0 {
        return Err(crate::ErrorCode::HealthFactorLessThanOne.into());
    }

    Ok(())
}
pub fn burn_tokens<'info>(
    amount_to_burn: u64,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    program: AccountInfo<'info>,
) -> Result<()> {
    let accounts = Burn {
        mint,
        from,
        authority,
    };
    let cpi_ctx = CpiContext::new(program, accounts);
    burn(cpi_ctx, amount_to_burn)
}

pub fn redeem_or_withdraw_collateral<'info>(
    to_redeem_amount: u64,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    program: AccountInfo<'info>,
) -> Result<()> {
    let accounts = Transfer { from, to };
    let cpi_ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
    transfer(cpi_ctx, to_redeem_amount)
}
