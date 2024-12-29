use crate::constants::{CONFIG_ACCOUNT, MINT_ACCOUNT, MINT_DECIMALS};
use crate::states::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022};
#[derive(Accounts)]
#[instruction(timestamp:i64)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        seeds =[MINT_ACCOUNT, &timestamp.to_le_bytes()],
        bump,
        mint::authority = mint_account,
        mint::decimals = MINT_DECIMALS,
        mint::freeze_authority = mint_account,
        mint::token_program = token_program
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_ACCOUNT, mint_account.key().as_ref(), &timestamp.to_le_bytes()],
        bump,
    )]
    pub config_account: Account<'info, Config>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

pub fn init_config_handler(
    ctx: Context<InitConfig>,
    timestamp: i64,
    max_ltv: u64,
    liquidation_threshold: u64,
    liquidation_bonus: u64,
    min_health_factor: u64,
) -> Result<()> {
    msg!("timestamp:{}", timestamp);
    *ctx.accounts.config_account = Config {
        authority: ctx.accounts.authority.key(),
        mint_account: ctx.accounts.mint_account.key(),
        max_ltv,
        liquidation_threshold,
        liquidation_bonus,
        min_health_factor,
        self_bump: ctx.bumps.config_account,
        mint_account_bump: ctx.bumps.mint_account,
        init_time: timestamp,
        last_update_time: timestamp,
    };
    Ok(())
}
