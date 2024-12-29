use crate::constants::{CONFIG_ACCOUNT, DEPOSIT_ASSET_ACCOUNT, MINT_ACCOUNT};
use crate::errors::ErrorCode;
use crate::states::{Collateral, Config};
use crate::utils::{burn_tokens, calc_health_factor_when_liquidate};
use crate::{calc_liquidatable_collateral, redeem_or_withdraw_collateral};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

///清算流程：清算人burn自己账户上的稳定币，获得被抵押资产sol和一定比例的清算奖励
///先检查健康因子判断是否可执行清算
///参数为amount_to_burn，通过pyth计算等值的被抵押资产sol，
#[derive(Accounts)]
#[instruction(timestamp:i64)]
pub struct Liquidation<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    // pub price_update: Account<'info, PriceUpdateV2>,
    /// CHECK: The account's data is validated manually within the handler.
    pub price_update: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [MINT_ACCOUNT, &timestamp.to_le_bytes()],
        bump
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [CONFIG_ACCOUNT, mint_account.key().as_ref(), &timestamp.to_le_bytes()],
        bump,
        has_one = mint_account,
    )]
    pub config_account: Account<'info, Config>,
    #[account(
        mut,
        has_one = deposited_asset_account
    )]
    pub collateral_account: Account<'info, Collateral>,
    #[account(
        mut,
        seeds = [DEPOSIT_ASSET_ACCOUNT, collateral_account.depositor.as_ref(), &timestamp.to_le_bytes()],
        bump
    )]
    pub deposited_asset_account: SystemAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub receive_stablecoin_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

pub fn liquidator_handler(
    ctx: Context<Liquidation>,
    timestamp:i64,
    amount_to_burn: u64,
) -> Result<()> {
    let collateral_account = &mut ctx.accounts.collateral_account;
    let price_update_account = &ctx.accounts.price_update.to_account_info();
    let price_update_data = price_update_account.try_borrow_data()?;
    let mut price_update_data = price_update_data.iter().as_slice();
    let price_update = PriceUpdateV2::try_deserialize(&mut price_update_data)?;
    let health_factor = calc_health_factor_when_liquidate(
        &price_update,
        collateral_account.deposited_asset_lamports,
        collateral_account.stablecoin_minted_amount,
        ctx.accounts.config_account.liquidation_threshold,
    )
    .expect("Invoke method calc_health_factor_when_liquidate encountered error!");
    let configured_min_health_factor = ctx.accounts.config_account.min_health_factor;
    msg!("health factor when liquidate:{}", health_factor);
    if health_factor >= configured_min_health_factor as f64 / 100.0 {
        return Err(ErrorCode::HealthFactorGreaterMinHealthFactor.into());
    }
    msg!("health checked completed!");
    burn_tokens(
        amount_to_burn,
        ctx.accounts.mint_account.to_account_info(),
        ctx.accounts.receive_stablecoin_account.to_account_info(),
        ctx.accounts.liquidator.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    )?;
    msg!("burn_tokens completed!");
    let liquidate_bonus = ctx.accounts.config_account.liquidation_bonus as f64 / 100.0;
    let liquidatable_amount = calc_liquidatable_collateral(&price_update, amount_to_burn)
        .expect("Invoke method calc_liquidatable_collateral encountered error!");
    let liquidatable_amount = (1.0 + liquidate_bonus) * liquidatable_amount as f64;
    let signer_seeds: &[&[&[u8]]] = &[&[
        DEPOSIT_ASSET_ACCOUNT,
        collateral_account.depositor.as_ref(),
        &timestamp.to_le_bytes(),
        &[collateral_account.deposited_asset_account_bump],
    ]];
    redeem_or_withdraw_collateral(
        liquidatable_amount as u64,
        ctx.accounts.deposited_asset_account.to_account_info(),
        ctx.accounts.liquidator.to_account_info(),
        signer_seeds,
        ctx.accounts.system_program.to_account_info(),
    )?;
    msg!("withdraw collateral completed!");
    collateral_account.deposited_asset_lamports -= liquidatable_amount as u64;
    collateral_account.stablecoin_minted_amount -= amount_to_burn;
    collateral_account.last_update_time = Clock::get()?.unix_timestamp;
    msg!("update collateral account completed!");

    Ok(())
}
