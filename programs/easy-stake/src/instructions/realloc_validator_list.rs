use anchor_lang::{prelude::*, system_program::ID as sys_id};

use crate::{
    error::StakingError, 
    state::{
        validator_system::{ValidatorList, ValidatorSystem}, 
        StakePoolConfig
    }
};


#[event]
pub struct ReallocValidatorListEvent {
    pub state: Pubkey,
    pub count: u32,
    pub new_capacity: u32,
}


#[derive(Accounts)]
#[instruction(capacity: u32)]
pub struct ReallocValidatorList<'info> {
    #[account(
        mut,
        owner = sys_id
    )]
    pub rent_funds: Signer<'info>,

    pub admin_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump,
        has_one = admin_authority @ StakingError::InvalidAdminAuthority
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        address = stake_pool_config.validator_system.validator_list.account,
        realloc = 8 + (ValidatorSystem::VALIDATOR_RECORD_LEN * capacity as usize),
        realloc::payer = rent_funds,
        realloc::zero = false
    )]
    pub validator_list: Account<'info, ValidatorList>,

    pub system_program: Program<'info, System>
}


impl<'info> ReallocValidatorList<'info> {
    pub fn process(&mut self, capacity: u32) -> Result<()> {
        require_gte!(
            capacity,
            self.stake_pool_config.validator_system.validator_list.count,
            StakingError::ShrinkingListWithDeletingContents
        );

        emit!(ReallocValidatorListEvent {
            state: self.stake_pool_config.key(),
            count: self.stake_pool_config.validator_system.validator_list.count,
            new_capacity: capacity
        });

        Ok(())
    }
}
