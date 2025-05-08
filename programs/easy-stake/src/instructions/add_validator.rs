//! 添加验证者

use anchor_lang::{prelude::*, system_program::ID as sys_id};

use crate::{
    ID,
    error::StakingError,
    state::{
        validator_system::{ValidatorList, ValidatorRecord}, 
        StakePoolConfig
    }
};


#[derive(Accounts)]
pub struct AddValidator<'info> {
    #[account(
        mut,
        owner = sys_id
    )]
    pub rent_payer: Signer<'info>,

    #[account(
        address = stake_pool_config.validator_system.manager_authority
    )]
    pub manager_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        owner = ID,
        address = stake_pool_config.validator_system.validator_list.account
    )]
    pub validator_list: Box<Account<'info, ValidatorList>>,

    /// CHECK: 验证者的投票账户（vote account）
    //只用于获取 Pubkey 不进行反序列化；用于标识将被加入的验证者节点。
    pub validator_vote: UncheckedAccount<'info>,

    /// CHECK: 唯一性哨兵账户，用于防止重复添加同一验证者。
    // 若该 PDA 已存在，则表示该验证者已被添加过；由 init 和 seeds 自动校验。
    #[account(
        init,
        payer = rent_payer,
        space = 0,
        seeds = [
            stake_pool_config.key().as_ref(),
            ValidatorRecord::DUPLICATE_FLAG_SEED,
            validator_vote.key().as_ref()
        ],
        bump
    )]
    pub duplication_flag: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>
}


impl<'info> AddValidator<'info> {
    pub fn process(&mut self, score: u32) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        msg!("Add validator {}", self.validator_vote.key);

        let stake_config_key = self.stake_pool_config.key();
        self.stake_pool_config.validator_system.add(
            &mut self.validator_list.to_account_info().data.borrow_mut(), 
            self.validator_vote.key(), 
            score, 
            &stake_config_key, 
            self.duplication_flag.key
        )?;

        Ok(())
    }
}
