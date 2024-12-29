pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use errors::ErrorCode;
pub use instructions::*;
pub use states::*;
pub use utils::*;

declare_id!("H4zjGkpfUbpzogZ7nxcBvvzvgQooQWt2mF2TxpLkXneQ");

#[program]
pub mod stablecoin {
    use super::*;

    pub fn process_init_config(
        ctx: Context<InitConfig>,
        timestamp: i64,
        max_ltv: u64,
        liquidation_threshold: u64,
        liquidation_bonus: u64,
        min_health_factor: u64,
    ) -> Result<()> {
        init_config_handler(
            ctx,
            timestamp,
            max_ltv,
            liquidation_threshold,
            liquidation_bonus,
            min_health_factor,
        )
    }

    pub fn process_update_config(
        ctx: Context<UpdateConfig>,
        timestamp: i64,
        liquidation_threshold: Option<u64>,
        liquidation_bonus: Option<u64>,
        min_health_factor: Option<u64>
    ) -> Result<()> {
        update_config_handler(
            ctx,
            timestamp,
            liquidation_threshold,
            liquidation_bonus,
            min_health_factor
        )
    }

    pub fn process_deposit_and_mint(
        ctx: Context<DepositCollateralAndMintTokens>,
        timestamp: i64,
        amount_to_deposit: u64,
    ) -> Result<()> {
        deposit_collateral_and_mint_tokens_handler(ctx,timestamp, amount_to_deposit)
    }

    pub fn process_burn_and_redeem(
        ctx: Context<BurnTokensAndRedeemCollateral>,
        timestamp: i64,
        amount_to_bun: u64,
    ) -> Result<()> {
        burn_and_redeem_handler(ctx, timestamp, amount_to_bun)
    }

    pub fn process_liquidate(
        ctx: Context<Liquidation>,
        timestamp: i64,
        amount_to_burn: u64,
    ) -> Result<()> {
        liquidator_handler(ctx, timestamp, amount_to_burn)
    }
}
