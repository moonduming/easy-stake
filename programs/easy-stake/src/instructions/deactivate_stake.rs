//! 解质押(验证者需提前从验证者列表删除)

use anchor_lang::{
    prelude::*, 
    system_program::ID as sys_id,
    solana_program::{
        sysvar::stake_history::ID as STAKE_HISTORY_ID,
        stake::{
            program::ID as STAKE_ID,
            state::StakeStateV2
        },
    }
};
use anchor_spl::stake::{StakeAccount, Stake};

use crate::{
    checks::check_stake_amount_and_validator, error::StakingError, state::{
        stake_system::{StakeList, StakeSystem}, 
        StakePoolConfig
    }
};


#[derive(Accounts)]
pub struct DeacitvateStake<'info> {
    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump,
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            StakePoolConfig::RESERVE_SEED
        ],
        bump = stake_pool_config.reserve_bump_seed
    )]
    pub reserve_pda: SystemAccount<'info>,

    /// CHECK: CPI
    #[account(mut)]
    pub validator_vote: UncheckedAccount<'info>,

    #[account(
        mut,
        address = stake_pool_config.stake_system.stake_list.account
    )]
    pub stake_list: Account<'info, StakeList>,

    #[account(mut)]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            StakeSystem::STAKE_DEPOSIT_SEED
        ],
        bump = stake_pool_config.stake_system.stake_deposit_bump_seed
    )]
    pub stake_deposit_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        owner = sys_id
    )]
    pub split_stake_rent_payer: Signer<'info>,

    #[account(
        init,
        payer = split_stake_rent_payer,
        space = std::mem::size_of::<StakeStateV2>(),
        owner = STAKE_ID
    )]
    pub split_stake_account: Account<'info, StakeAccount>,

    /// CHECK: have no CPU budget to parse
    #[account(address = STAKE_HISTORY_ID)]
    pub stake_history: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub epoch_schedule: Sysvar<'info, EpochSchedule>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub stake_program: Program<'info, Stake>,
}

impl<'info> DeacitvateStake<'info> {
    pub fn process(
        &mut self,
        stake_index: u32
    ) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        let stake = self.stake_pool_config.stake_system.get_checked(
            &self.stake_list.to_account_info().data.borrow(), 
            stake_index, 
            self.stake_account.to_account_info().key
        )?;

        check_stake_amount_and_validator(
            &self.stake_account, 
            stake.last_update_delegated_lamports, 
            self.validator_vote.key
        )?;
        
        // 将stake_account中的钱转入reserve_pda
        // 销毁stake_account

        Ok(())
    }
}
