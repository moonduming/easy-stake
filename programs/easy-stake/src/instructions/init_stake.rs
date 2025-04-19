//! 用户账户初始化指令

use anchor_lang::prelude::*;

use crate::{state::{ConfigAccount, StakingAccount}, constants::DISCRIMINATOR_LENGTH};


#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct InitStake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = DISCRIMINATOR_LENGTH + StakingAccount::INIT_SPACE,
        seeds = [
            b"staking".as_ref(),
            pool_id.to_le_bytes().as_ref(),
            authority.key.as_ref()
        ],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
        seeds = [b"config".as_ref(), pool_id.to_le_bytes().as_ref()],
        bump
    )]
    pub config_account: Account<'info, ConfigAccount>,


    pub system_program: Program<'info, System>,
}


pub fn process_init_stake(ctx: Context<InitStake>, _pool_id: u64, lock_start: u64, lock_period: u64) -> Result<()> {
    let staking_account = &mut ctx.accounts.staking_account;
    staking_account.authority = ctx.accounts.authority.key();
    staking_account.reward_vault = ctx.accounts.config_account.reward_vault;
    staking_account.stake_amount = 0;
    staking_account.lock_start = lock_start;
    staking_account.lock_period = lock_period;
    staking_account.bump = ctx.bumps.staking_account;

    Ok(())
}
