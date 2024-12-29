use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Config {
    //官方机构管理员
    pub authority: Pubkey,
    //铸币地址
    pub mint_account: Pubkey,
    //抵押物最大抵押率
    pub max_ltv: u64,
    //清算阈值
    pub liquidation_threshold: u64,
    //清算奖励
    pub liquidation_bonus: u64,
    //最低健康因子
    pub min_health_factor: u64,
    pub self_bump: u8,
    pub mint_account_bump: u8,
    pub init_time: i64,
    pub last_update_time: i64,
}
