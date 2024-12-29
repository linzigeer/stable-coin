use crate::constants::{COLLATERAL_ACCOUNT, CONFIG_ACCOUNT, DEPOSIT_ASSET_ACCOUNT, MINT_ACCOUNT};
use crate::redeem_or_withdraw_collateral;
use crate::states::{Collateral, Config};
use crate::utils::{
    burn_tokens, calc_redeemable_amount,
    check_health_factor_when_burn_tokens_and_redeem_collateral, get_collateral_in_usd,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
#[instruction(timestamp: i64)]
pub struct BurnTokensAndRedeemCollateral<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        seeds = [MINT_ACCOUNT, &timestamp.to_le_bytes()],
        bump
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [COLLATERAL_ACCOUNT, depositor.key().as_ref(), &timestamp.to_le_bytes()],
        bump = collateral_account.self_bump,
        has_one = deposited_asset_account,
        has_one = receive_stablecoin_account,
    )]
    pub collateral_account: Account<'info, Collateral>,
    #[account(
        seeds = [CONFIG_ACCOUNT, mint_account.key().as_ref(), &timestamp.to_le_bytes()],
        bump = config_account.self_bump,
        has_one = mint_account,
    )]
    pub config_account: Account<'info, Config>,
    #[account(
        mut,
        seeds = [DEPOSIT_ASSET_ACCOUNT, depositor.key().as_ref(), &timestamp.to_le_bytes()],
        bump
    )]
    pub deposited_asset_account: SystemAccount<'info>,
    #[account(mut)]
    pub receive_stablecoin_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: The account's data is validated manually within the handler.
    pub price_update: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn burn_and_redeem_handler(
    ctx: Context<BurnTokensAndRedeemCollateral>,
    timestamp: i64,
    amount_to_burn: u64,
) -> Result<()> {
    let collateral_account = &mut ctx.accounts.collateral_account;
    let key = ctx.accounts.depositor.key();
    let bump = collateral_account.deposited_asset_account_bump;
    msg!("1 deposited_asset_account_bump:{}", bump);
    let signer_seeds: &[&[&[u8]]] = &[&[
        DEPOSIT_ASSET_ACCOUNT,
        key.as_ref(),
        &timestamp.to_le_bytes(),
        &[bump],
    ]];
    msg!(
        "burn and redeem deposited_asset_account signer_seeds:{:?}",
        signer_seeds
    );

    let config_account = &ctx.accounts.config_account;
    // let price_update = &ctx.accounts.price_update;
    let price_update_account = &ctx.accounts.price_update.to_account_info();
    let price_update_data = price_update_account.try_borrow_data()?;
    let mut price_update_data = price_update_data.iter().as_slice();
    let price_update = PriceUpdateV2::try_deserialize(&mut price_update_data)?;

    let collateral_in_usd = get_collateral_in_usd(&price_update)?;

    let redeemable_amount = calc_redeemable_amount(amount_to_burn, collateral_in_usd)
        .expect("Invoke method calc_redeemable_amount encountered unexpected error!");
    let configured_min_health_factor = ctx.accounts.config_account.min_health_factor;
    check_health_factor_when_burn_tokens_and_redeem_collateral(
        &price_update,
        collateral_account.deposited_asset_lamports,
        collateral_account.stablecoin_minted_amount,
        redeemable_amount,
        amount_to_burn,
        config_account.liquidation_threshold,
        configured_min_health_factor,
    )?;
    msg!("Health check passed!");
    burn_tokens(
        amount_to_burn,
        ctx.accounts.mint_account.to_account_info(),
        ctx.accounts.receive_stablecoin_account.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    )?;
    msg!("burn token completed!");
    redeem_or_withdraw_collateral(
        redeemable_amount,
        ctx.accounts.deposited_asset_account.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        signer_seeds,
        ctx.accounts.system_program.to_account_info(),
    )?;
    msg!("redeem collateral completed!");
    collateral_account.deposited_asset_lamports =
        **ctx.accounts.deposited_asset_account.lamports.borrow() - redeemable_amount;
    collateral_account.stablecoin_minted_amount =
        ctx.accounts.receive_stablecoin_account.amount - amount_to_burn;
    collateral_account.last_update_time = Clock::get()?.unix_timestamp;
    msg!("update collateral_account completed!");

    Ok(())
}
