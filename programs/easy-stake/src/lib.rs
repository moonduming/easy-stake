//! 程序入口

use anchor_lang::prelude::*;

pub mod state;
pub mod checks;
pub mod error;
pub mod instructions;

use instructions::*;


declare_id!("J8iXwM3SQQpL4PhQ2wXZBWfZ7oFmNRdFZHnHHSr2yiUd");

#[program]
pub mod easy_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: InitializeData) -> Result<()> {
        ctx.accounts.process(data, ctx.bumps.reserve_pda)
    }
}

