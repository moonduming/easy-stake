//! 移除验证者

use anchor_lang::{prelude::*, system_program::ID as sys_id};

use crate::{
    ID,
    error::StakingError, 
    state::{
        validator_system::{ValidatorList, ValidatorRecord}, 
        StakePoolConfig
    }
};


#[event]
pub struct RemoveValidatorEvent {
    pub state: Pubkey,
    pub validator: Pubkey,
    pub index: u32,
    pub operational_sol_balance: u64,
}


#[derive(Accounts)]
#[instruction(validator_vote: Pubkey)]
pub struct RemoveValidator<'info> {
    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump,
        has_one = operational_sol_account
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    /// CHECK: not important
    #[account(mut)]
    pub operational_sol_account: UncheckedAccount<'info>,

    #[account(
        address = stake_pool_config.validator_system.manager_authority 
            @ StakingError::InvalidValidatorManager
    )]
    pub manager_authority: Signer<'info>,

    #[account(
        mut,
        address = stake_pool_config.validator_system.validator_list.account
    )]
    pub validator_list: Account<'info, ValidatorList>,

    /// CHECK: none
    #[account(
        mut,
        owner = ID,
        rent_exempt = enforce,
        seeds = [
            stake_pool_config.key().as_ref(),
            ValidatorRecord::DUPLICATE_FLAG_SEED,
            validator_vote.as_ref()
        ],
        bump
    )]
    pub duplication_flag: UncheckedAccount<'info>
}


impl<'info> RemoveValidator<'info> {
    pub fn process(&mut self, index: u32, validator_vote: Pubkey) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        let validator = self.stake_pool_config.validator_system.get_checked(
            &self.validator_list.to_account_info().data.borrow(), 
            index, 
            validator_vote
        )?;

        require_keys_eq!(
            self.duplication_flag.key(),
            validator.duplication_flag_address(
                &self.stake_pool_config.key()
            ),
            StakingError::WrongValidatorDuplicationFlag
        );

        self.stake_pool_config.validator_system.remove(
            &mut self.validator_list.to_account_info().data.borrow_mut(), 
            index, 
            validator
        )?;

        let operational_sol_balance = self.operational_sol_account.lamports();
        let rent_return = self.duplication_flag.lamports();
        **self.duplication_flag.try_borrow_mut_lamports()? = 0;
        **self.operational_sol_account.try_borrow_mut_lamports()? += rent_return;

        self.duplication_flag.assign(&sys_id);

        emit!(RemoveValidatorEvent {
            state: self.stake_pool_config.key(),
            validator: validator_vote,
            index,
            operational_sol_balance,
        });
        
        Ok(())
    }
}
