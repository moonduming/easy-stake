//! 程序入口

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod constants;

use instructions::*;

declare_id!("J8iXwM3SQQpL4PhQ2wXZBWfZ7oFmNRdFZHnHHSr2yiUd");

#[program]
pub mod easy_stake {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, pool_id: u64, reward_rate: u64) -> Result<()> {
        process_init_config(ctx, pool_id, reward_rate)
    }

    pub fn init_stake(
        ctx: Context<InitStake>, 
        pool_id: u64, 
        lock_start: u64, 
        lock_period: u64
    ) -> Result<()> {
        process_init_stake(ctx, pool_id, lock_start, lock_period)
    }
}

