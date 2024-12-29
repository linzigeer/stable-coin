use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Collateral {
    //资产抵押人
    pub depositor: Pubkey,
    //抵押资产账户
    pub deposited_asset_account: Pubkey,
    //接收稳定币账户
    pub receive_stablecoin_account: Pubkey,
    //抵押资产的lamports数量
    pub deposited_asset_lamports: u64,
    //稳定币铸造数量
    pub stablecoin_minted_amount: u64,
    pub self_bump: u8,
    pub deposited_asset_account_bump: u8,
    //是否初始化标志，避免意外覆盖某些值
    pub is_initialized: bool,
    pub init_time: i64,
    pub last_update_time: i64,
}
