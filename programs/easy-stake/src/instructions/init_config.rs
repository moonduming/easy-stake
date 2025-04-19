//! 全局账户初始化指令

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Token}
  };

use crate::{state::ConfigAccount, constants::DISCRIMINATOR_LENGTH};


/// 全局账户上下文结构体
#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct InitConfig<'info> {
    /// 权限账户
    #[account(mut)]
    pub authority: Signer<'info>,

    /// 报酬账户 token mint 地址
    pub reward_mint: Account<'info, Mint>,
    /// 质押账户 token mint 地址
    pub stake_mint: Account<'info, Mint>,

    /// 配置账户
    #[account(
        init,
        payer = authority,
        space = DISCRIMINATOR_LENGTH + ConfigAccount::INIT_SPACE,
        seeds = [b"config".as_ref(), pool_id.to_le_bytes().as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,

    /// 奖励账户
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = reward_mint,
        associated_token::authority = config_account,
        associated_token::token_program = token_program
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    /// 质押token收取账户
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = stake_mint,
        associated_token::authority = config_account,
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    /// 程序账户
    pub system_program: Program<'info, System>,
    /// token交易账户
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// token程序
    pub token_program: Program<'info, Token>
}


/// 全局账户初始化指令
pub fn process_init_config(ctx: Context<InitConfig>, _pool_id: u64, reward_rate: u64) -> Result<()> {
    let config_account = &mut ctx.accounts.config_account;
    config_account.reward_vault = ctx.accounts.reward_vault.key();
    config_account.stake_vault = ctx.accounts.stake_vault.key();
    config_account.reward_rate = reward_rate;

    Ok(())
}
