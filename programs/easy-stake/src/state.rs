//! 账户字段配置文件

use anchor_lang::prelude::*;


/// 用户质押账户
#[account]
#[derive(InitSpace)]
pub struct StakingAccount {
    /// 用户
    pub authority: Pubkey,
    /// 奖励发放地址
    pub reward_vault: Pubkey,
    /// 存入的token数量
    pub stake_amount: u64,
    /// 质押开始时间
    pub lock_start: u64,
    /// 锁定时长
    pub lock_period: u64,
    /// 种子
    pub bump: u8
}


/// 全局配置账户，存储奖励池地址和全局速率
#[account]
#[derive(InitSpace)]
pub struct ConfigAccount {
    /// 回报代币的 Vault 地址（TokenAccount PDA）
    pub reward_vault: Pubkey,
    /// 质押token存入地址
    pub stake_vault: Pubkey,
    /// 每秒发放的奖励速率
    pub reward_rate: u64,
}
