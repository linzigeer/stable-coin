use crate::constants::{COLLATERAL_ACCOUNT, CONFIG_ACCOUNT, DEPOSIT_ASSET_ACCOUNT, MINT_ACCOUNT};
use crate::states::{Collateral, Config};
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
#[instruction(timestamp: i64)]
pub struct DepositCollateralAndMintTokens<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        seeds = [CONFIG_ACCOUNT, mint_account.key().as_ref(), &timestamp.to_le_bytes()],
        bump = config_account.self_bump,
        has_one = mint_account
    )]
    pub config_account: Account<'info, Config>,
    #[account(
        mut,
        seeds = [DEPOSIT_ASSET_ACCOUNT, depositor.key().as_ref(), &timestamp.to_le_bytes()],
        bump
    )]
    pub deposited_asset_account: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = mint_account,
        associated_token::authority = depositor,
        associated_token::token_program = token_program
    )]
    pub receive_stablecoin_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = depositor,
        seeds = [COLLATERAL_ACCOUNT, depositor.key().as_ref(), &timestamp.to_le_bytes()],
        space = 8 + Collateral::INIT_SPACE,
        bump,
    )]
    pub collateral_account: Account<'info, Collateral>,
    #[account(
        mut,
        seeds = [MINT_ACCOUNT, &timestamp.to_le_bytes()],
        bump,
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    // pub price_update: Account<'info, PriceUpdateV2>,
    // #[account(
    //     owner = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ"
    // )]
    // pub price_update: Account<'info, PriceUpdateV2>,
    /// CHECK: The account's data is validated manually within the handler.
    pub price_update: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn deposit_collateral_and_mint_tokens_handler(
    ctx: Context<DepositCollateralAndMintTokens>,
    timestamp: i64,
    amount_to_deposit: u64,
) -> Result<()> {
    let collateral_account = &mut ctx.accounts.collateral_account;
    if !collateral_account.is_initialized {
        collateral_account.is_initialized = true;
        collateral_account.depositor = ctx.accounts.depositor.key();
        collateral_account.deposited_asset_account = ctx.accounts.deposited_asset_account.key();
        collateral_account.receive_stablecoin_account =
            ctx.accounts.receive_stablecoin_account.key();
        collateral_account.self_bump = ctx.bumps.collateral_account;
        collateral_account.deposited_asset_account_bump = ctx.bumps.deposited_asset_account;
        collateral_account.init_time = timestamp;
    }
    let key = ctx.accounts.depositor.key();
    let bump = collateral_account.deposited_asset_account_bump;
    msg!("0 deposited_asset_account_bump:{}",bump);
    let signer_seeds: &[&[&[u8]]] = &[&[
        DEPOSIT_ASSET_ACCOUNT,
        key.as_ref(),
        &timestamp.to_le_bytes(),
        &[bump],
    ]];
    msg!("deposit and mint deposited_asset_account signer_seeds:{:?}", signer_seeds);
    
    let price_update_account = &ctx.accounts.price_update.to_account_info();
    let price_update_data = price_update_account.try_borrow_data()?;
    let mut price_update_data = price_update_data.iter().as_slice();
    let price_update = PriceUpdateV2::try_deserialize(&mut price_update_data)?;

    let collateral_in_usd = get_collateral_in_usd(&price_update)?;
    let mintable_amount = calc_mintable_amount(
        amount_to_deposit,
        collateral_in_usd,
        ctx.accounts.config_account.max_ltv,
    )
    .expect("invoke method calc_mintable_amount encounter error!");
    let configured_min_health_factor = ctx.accounts.config_account.min_health_factor;
    check_health_factor_when_deposit_collateral_and_mint_new_tokens(
        &price_update,
        **ctx.accounts.deposited_asset_account.lamports.borrow(),
        ctx.accounts.receive_stablecoin_account.amount,
        amount_to_deposit,
        mintable_amount,
        ctx.accounts.config_account.liquidation_threshold,
        configured_min_health_factor,
    )?;

    deposit_collateral(
        amount_to_deposit,
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.deposited_asset_account.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    let signer_seeds: &[&[&[u8]]] = &[&[
        MINT_ACCOUNT,
        &timestamp.to_le_bytes(),
        &[ctx.bumps.mint_account],
    ]];
    mint_stable_coins(
        mintable_amount,
        ctx.accounts.mint_account.to_account_info(),
        ctx.accounts.receive_stablecoin_account.to_account_info(),
        ctx.accounts.mint_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        signer_seeds,
    )?;

    collateral_account.stablecoin_minted_amount =
        mintable_amount + ctx.accounts.receive_stablecoin_account.amount;
    collateral_account.deposited_asset_lamports =
        amount_to_deposit + **ctx.accounts.deposited_asset_account.lamports.borrow();
    collateral_account.last_update_time = Clock::get()?.unix_timestamp;

    Ok(())
}
