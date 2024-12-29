use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::{Collateral, Config};

#[derive(Accounts)]
pub struct RedeemCollateralAndBurnTokens<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub mint_account:InterfaceAccount<'info, Mint>,
    
    pub collateral_account:Account<'info, Collateral>,
    
    pub config_account:Account<'info, Config>,
    
    pub deposited_asset_account:SystemAccount<'info>,
    
    pub stablecoin_minted_amount:InterfaceAccount<'info, TokenAccount>,
}
