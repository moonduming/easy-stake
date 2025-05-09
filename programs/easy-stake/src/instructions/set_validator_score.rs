//! 修改验证者分数

use anchor_lang::prelude::*;

use crate::{error::StakingError, state::{validator_system::ValidatorList, StakePoolConfig}};


#[event]
pub struct SetValidatorScoreEvent {
    pub state: Pubkey,
    pub validator: Pubkey,
    pub index: u32,
    pub old_score: u32,
    pub new_score: u32
}


#[derive(Accounts)]
pub struct SetValidatorScore<'info> {
    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump,
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        address = stake_pool_config.validator_system.manager_authority 
            @ StakingError::InvalidValidatorManager
    )]
    pub manager_authority: Signer<'info>,

    #[account(
        mut,
        address = stake_pool_config.validator_system.validator_list.account
    )]
    pub validator_list: Account<'info, ValidatorList>
}


impl<'info> SetValidatorScore<'info> {
    pub fn process(
        &mut self,
        index: u32,
        validator_vote: Pubkey,
        score: u32
    ) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        let mut validator = self.stake_pool_config.validator_system.get_checked(
            &self.validator_list.to_account_info().data.borrow(), 
            index, 
            validator_vote
        )?;

        self.stake_pool_config.validator_system.total_validator_score -= validator.score;

        let old = validator.score;
        validator.score = score;

        self.stake_pool_config.validator_system.total_validator_score += score;
        self.stake_pool_config.validator_system.set(
            &mut self.validator_list.to_account_info().data.borrow_mut(),
            index,
            validator
        )?;

        emit!(SetValidatorScoreEvent {
            state: self.stake_pool_config.key(),
            validator: validator_vote,
            index,
            old_score: old,
            new_score: score
        });
        
        Ok(())
    }
}
