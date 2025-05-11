//! 质押sol到验证节点

use anchor_lang::{
    prelude::*, 
    solana_program::{
        log::sol_log_compute_units, 
        program::{invoke, invoke_signed}, 
        stake::{
            self,
            config::ID as STAKE_CONFIG_ID, program::ID as STAKE_ID, 
            state::{Authorized, Lockup, StakeStateV2}
        }, 
        sysvar::stake_history::ID as STAKE_HISTORY_ID
    }, 
    system_program::{transfer, Transfer, ID as sys_id}
};


use anchor_spl::stake::{withdraw, Stake, StakeAccount, Withdraw};

use crate::{
    ID,
    error::StakingError,
    state::{
        stake_system::{StakeList, StakeSystem}, 
        validator_system::ValidatorList, 
        StakePoolConfig
    }
};


#[event]
pub struct StakeReserveEvent {
    pub state: Pubkey,
    pub epoch: u64,
    pub stake_index: u32,
    pub stake_account: Pubkey,
    pub validator_index: u32,
    pub validator_vote: Pubkey,
    pub stake_target: u64,
    pub validator_stake_target: u64,
    pub reserve_balance: u64,
    pub total_active_balance: u64,
    pub validator_active_balance: u64,
    pub stake_delta: u64,
    pub amount: u64,
}


#[derive(Accounts)]
pub struct StakeReserve<'info> {
    #[account(
        mut,
        owner = sys_id
    )]
    pub rent_payer: Signer<'info>,

    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump,
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        address = stake_pool_config.validator_system.validator_list.account
    )]
    pub validator_list: Account<'info, ValidatorList>,

    #[account(
        mut,
        address = stake_pool_config.stake_system.stake_list.account
    )]
    pub stake_list: Account<'info, StakeList>,

    /// CHECK: CPI
    #[account(mut)]
    pub validator_vote: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            stake_pool_config.key().as_ref(),
            StakePoolConfig::RESERVE_SEED
        ],
        bump = stake_pool_config.reserve_bump_seed
    )]
    pub reserve_pda: SystemAccount<'info>,

    #[account(
        init,
        payer = rent_payer,
        space = std::mem::size_of::<StakeStateV2>(),
        owner = STAKE_ID
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// CHECK: PDA
    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            StakeSystem::STAKE_DEPOSIT_SEED
        ],
        bump = stake_pool_config.stake_system.stake_deposit_bump_seed
    )]
    pub stake_deposit_authority: UncheckedAccount<'info>,

    /// CHECK: have no CPU budget to parse
    #[account(address = STAKE_HISTORY_ID)]
    pub stake_history: UncheckedAccount<'info>,

    /// CHECK: CPI
    #[account(address = STAKE_CONFIG_ID)]
    pub stake_config: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub epoch_schedule: Sysvar<'info, EpochSchedule>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub stake_program: Program<'info, Stake>,
}


impl<'info> StakeReserve<'info> {
    pub fn process(&mut self, validator_index: u32) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        // 打印当前剩余 CU 数量
        sol_log_compute_units();

        let total_active_balance = self.stake_pool_config.validator_system.total_active_balance;

        let staker = Pubkey::create_program_address(
            &[
                self.stake_pool_config.key().as_ref(),
                StakeSystem::STAKE_DEPOSIT_SEED,
                &[self.stake_pool_config.stake_system.stake_deposit_bump_seed]
            ], 
            &ID
        ).unwrap();

        let withdrawer = Pubkey::create_program_address(
            &[
                self.stake_pool_config.key().as_ref(),
                StakeSystem::STAKE_WITHDRAW_SEED,
                &[self.stake_pool_config.stake_system.stake_withdraw_bump_seed]
            ], 
            &ID
        ).unwrap();

        let reserve_balance = self.reserve_pda.lamports();
        let stake_delta = self.stake_pool_config.stake_delta(reserve_balance);
        if stake_delta == 0 {
            return Ok(());
        }

        let mut validator = self.stake_pool_config.validator_system.get_checked(
            &self.validator_list.to_account_info().data.borrow(), 
            validator_index, 
            self.validator_vote.key()
        ).map_err(|e| e.with_account_name("validator_vote"))?;

        let validator_active_balance = validator.active_balance;
        if validator.last_stake_delta_epoch == self.clock.epoch {
            if self.stake_pool_config.stake_system.extra_stake_delta_runs == 0 {
                msg!(
                    "Double delta stake command for validator {} in epoch {}",
                    validator.validator_account,
                    self.clock.epoch
                );
                self.return_unused_stake_account_rent()?;
                return Ok(());
            } else {
                self.stake_pool_config.stake_system.extra_stake_delta_runs -= 1;
            }
        }

        let last_slot = self.epoch_schedule.get_last_slot_in_epoch(self.clock.epoch);

        require_gte!(
            self.clock.slot,
            last_slot.saturating_sub(
                self.stake_pool_config.stake_system.slots_for_stake_delta
            ),
            StakingError::TooEarlyForStakeDelta
        );

        let validator_stake_target = self.stake_pool_config.validator_system
            .validator_stake_target(
                &validator, 
                stake_delta
            )?;

        if validator_active_balance >= validator_stake_target {
            msg!(
                "Validator {} has already reached stake target {}. Please stake into another validator",
                validator.validator_account,
                validator_stake_target
            );
            self.return_unused_stake_account_rent()?;
            return Ok(());
        }

        let stake_target = validator_stake_target
            .saturating_sub(validator_active_balance)
            .min(stake_delta);

        let stake_target = if stake_delta - stake_target 
            < self.stake_pool_config.stake_system.min_stake 
        {
            stake_delta
        } else {
            stake_target
        };

        if stake_target < self.stake_pool_config.stake_system.min_stake {
            msg!(
                "Resulting stake {} is lower than min stake allowed {}",
                stake_target,
                self.stake_pool_config.stake_system.min_stake
            );
            self.return_unused_stake_account_rent()?;
            return Ok(()); // Not an error. Don't fail other instructions in tx
        }
        
        sol_log_compute_units();
        msg!("Transfer to stake account");
        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.reserve_pda.to_account_info(),
                    to: self.stake_account.to_account_info()
                }, 
                &[&[
                    self.stake_pool_config.key().as_ref(),
                    StakePoolConfig::RESERVE_SEED,
                    &[self.stake_pool_config.reserve_bump_seed]
                ]]
            ), 
            stake_target
        )?;
        self.stake_pool_config.on_transfer_to_reserve(stake_target);

        sol_log_compute_units();
        msg!("Initialize stake");
        invoke(
            &stake::instruction::initialize(
                &self.stake_account.key(), 
                &Authorized {staker, withdrawer}, 
                &Lockup::default(),
            ), 
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.rent.to_account_info()
            ]
        )?;

        sol_log_compute_units();
        msg!("Delegate stake");
        invoke_signed(
            &stake::instruction::delegate_stake(
                &self.stake_account.key(), 
                &staker, 
                self.validator_vote.key
            ), 
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.stake_deposit_authority.to_account_info(),
                self.validator_vote.to_account_info(),
                self.clock.to_account_info(),
                self.stake_history.to_account_info(),
                self.stake_config.to_account_info()
            ], 
            &[&[
                self.stake_pool_config.key().as_ref(),
                StakeSystem::STAKE_DEPOSIT_SEED,
                &[self.stake_pool_config.stake_system.stake_deposit_bump_seed]
            ]]
        )?;

        self.stake_pool_config.stake_system.add(
            &mut self.stake_list.to_account_info().data.borrow_mut(), 
            &self.stake_account.key(), 
            stake_target, 
            &self.clock, 
            0
        )?;

        validator.active_balance += stake_target;
        validator.last_stake_delta_epoch = self.clock.epoch;
        self.stake_pool_config.stake_system.last_stake_delta_epoch = self.clock.epoch;
        self.stake_pool_config.validator_system.set(
            &mut self.validator_list.to_account_info().data.borrow_mut(), 
            validator_index, 
            validator
        )?;

        self.stake_pool_config.validator_system.total_active_balance += stake_target;

        emit!(StakeReserveEvent {
            state: self.stake_pool_config.key(),
            epoch: self.clock.epoch,
            stake_index: self.stake_pool_config.stake_system.stake_list.count - 1,
            stake_account: self.stake_account.key(),
            validator_index,
            validator_vote: self.validator_vote.key(),
            amount: stake_target,
            stake_target,
            validator_stake_target,
            reserve_balance,
            total_active_balance,
            validator_active_balance,
            stake_delta,
        });

        Ok(())
    }

    pub fn return_unused_stake_account_rent(&self) -> Result<()> {
        withdraw(
            CpiContext::new(
                self.stake_program.to_account_info(), 
                Withdraw {
                    stake: self.stake_account.to_account_info(),
                    withdrawer: self.stake_account.to_account_info(),
                    to: self.rent_payer.to_account_info(),
                    clock: self.clock.to_account_info(),
                    stake_history: self.stake_history.to_account_info()
                }
            ), 
            self.stake_account.to_account_info().lamports(), 
            None
        )
    }
}
