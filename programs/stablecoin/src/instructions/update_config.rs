use crate::constants::{CONFIG_ACCOUNT, MINT_ACCOUNT};
use crate::states::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[derive(Accounts)]
#[instruction(timestamp:i64)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [MINT_ACCOUNT, &timestamp.to_le_bytes()],
        bump
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [CONFIG_ACCOUNT, mint_account.key().as_ref(), &timestamp.to_le_bytes()],
        bump = config_account.self_bump
    )]
    pub config_account: Account<'info, Config>,
}

pub fn update_config_handler(
    ctx: Context<UpdateConfig>,
    _timestamp: i64,
    liquidation_threshold: Option<u64>,
    liquidation_bonus: Option<u64>,
    min_health_factor: Option<u64>,
    
) -> Result<()> {
    let config_account = &mut ctx.accounts.config_account;

    if let Some(threshold) = liquidation_threshold {
        config_account.liquidation_threshold = threshold;
    }

    if let Some(bonus) = liquidation_bonus {
        config_account.liquidation_bonus = bonus;
    }

    if let Some(factor) = min_health_factor {
        config_account.min_health_factor = factor;
    }

    config_account.last_update_time = Clock::get()?.unix_timestamp;

    Ok(())
}
