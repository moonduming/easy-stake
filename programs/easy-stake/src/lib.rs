use anchor_lang::prelude::*;

pub mod state;

declare_id!("J8iXwM3SQQpL4PhQ2wXZBWfZ7oFmNRdFZHnHHSr2yiUd");

#[program]
pub mod easy_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
