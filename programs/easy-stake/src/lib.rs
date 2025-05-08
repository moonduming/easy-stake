//! 程序入口

use anchor_lang::{prelude::*, Bumps};

pub mod checks;
pub mod error;
pub mod instructions;
pub mod state;
pub mod calc;

use instructions::*;
use error::StakingError;

declare_id!("J8iXwM3SQQpL4PhQ2wXZBWfZ7oFmNRdFZHnHHSr2yiUd");


fn check_context<T>(ctx: &Context<T>) -> Result<()>
where T: Bumps {
    if !check_id(ctx.program_id) {
        return err!(StakingError::InvalidProgramId);
    }

    if !ctx.remaining_accounts.is_empty() {
        return err!(StakingError::UnexpectedAccount);
    }

    Ok(())
}

#[program]
pub mod easy_stake {
    use super::*;

    // 初始化
    pub fn initialize(ctx: Context<Initialize>, data: InitializeData) -> Result<()> {
        check_context(&ctx)?;     
        ctx.accounts.process(data, ctx.bumps)
    }

    // 用户质押
    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(lamports)
    }

    // 用户解质押
    pub fn unstake(ctx: Context<Unstake>, msol_amount: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(msol_amount)
    }

    // 质押池添加流动性(只能通过sol进行添加)
    pub fn add_liquidity(ctx: Context<AddLiquidity>, lamports: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(lamports)
    }

    // 提取质押池代币
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, tokens: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(tokens)
    }

    // 添加验证者
    pub fn add_validator(ctx: Context<AddValidator>, score: u32) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(score)
    }

    // 移除验证者
    pub fn remove_validator(
        ctx: Context<RemoveValidator>,
        index: u32,
        validator_vote: Pubkey
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(index, validator_vote)
    }
}
