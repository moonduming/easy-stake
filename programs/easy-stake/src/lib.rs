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

    pub fn initialize(ctx: Context<Initialize>, data: InitializeData) -> Result<()> {
        check_context(&ctx)?;     
        ctx.accounts.process(data, ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(lamports)
    }
}
