use anchor_lang::{
    prelude::*, 
    solana_program::{
        system_instruction, 
        program::invoke, 
        program_pack::Pack
    }
};
use anchor_spl::token::{self, spl_token::{self, instruction::AuthorityType}, Mint, SetAuthority, Token, TokenAccount};


use crate::{error::StakingError, require_lte, state::{Fee, LiqPool, StakePoolConfig, StakeSystem, ValidatorSystem}};


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// 初始化时的质押池配置账户，要求账户数据为零
    #[account(
        init,
        payer = payer,
        space = StakePoolConfig::serialized_len(),
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump,
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    ///CHECK: 质押池配置的储备PDA账户，使用种子和bump生成
    #[account(
        init,
        payer = payer,
        space = 0,
        seeds = [
            &stake_pool_config.key().to_bytes(),
            StakePoolConfig::RESERVE_SEED
        ],
        bump
    )]
    pub reserve_pda: UncheckedAccount<'info>,

    /// CHECK: 初始化时允许未校验账户，后续会在逻辑中验证 owner 和结构合法性
    pub stake_list: UncheckedAccount<'info>,

    /// CHECK: 此账户在初始化后将由逻辑手动校验其 owner 和数据内容
    pub validator_list: UncheckedAccount<'info>,

    #[account(
        seeds = [
            stake_pool_config.key().as_ref(), 
            StakePoolConfig::MSOL_MINT_AUTHORITY_SEED
        ],
        bump
    )]
    /// CHECK: PDA，用于铸造/销毁 mSOL
    pub msol_mint_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [
            stake_pool_config.key().as_ref(), 
            LiqPool::LP_MINT_AUTHORITY_SEED
        ],
        bump
    )]
    /// CHECK: PDA，用于铸造/销毁 lp token
    pub lp_mint_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [
            stake_pool_config.key().as_ref(), 
            LiqPool::MSOL_LEG_AUTHORITY_SEED
        ],
        bump
    )]
    /// CHECK: PDA，用于铸造/销毁 mSOL
    pub msol_leg_authority: UncheckedAccount<'info>,

    /// mSOL代币铸造账户
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_pool_config.key().as_ref(), 
            StakePoolConfig::MSOL_MINT_SEED
        ],
        bump,
        mint::decimals = 9,
        mint::authority = msol_mint_authority
    )]
    pub msol_mint: Box<Account<'info, Mint>>,

    /// LP代币铸造账户
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_pool_config.key().as_ref(), 
            LiqPool::LP_MINT_SEED
        ],
        bump,
        mint::decimals = 9,
        mint::authority = lp_mint_authority
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// 用于流动性池中存储 MSOL 的 PDA 账户
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_pool_config.key().as_ref(), 
            LiqPool::MSOL_LEG_SEED
        ],
        bump,
        token::mint = msol_mint,
        token::authority = msol_leg_authority
    )]
    pub msol_leg: Box<Account<'info, TokenAccount>>,

    /// 操作用的SOL账户
    pub operational_sol_account: SystemAccount<'info>,

    /// CHECK: 用于流动性池中存储 SOL 的 PDA 账户
    #[account(
        init,
        payer = payer,
        space = 0,
        seeds = [
            stake_pool_config.key().as_ref(),
            LiqPool::SOL_LEG_SEED
        ],
        bump
    )]
    pub sol_leg_pda: UncheckedAccount<'info>,

    /// 用于托管mSOL的Token账户，需与msol_mint对应
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_pool_config.key().as_ref(), 
            StakePoolConfig::TREASURY_MSOL_SEED
        ],
        bump,
        token::mint = msol_mint,
        token::authority = stake_pool_config
    )]
    pub treasury_msol_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}



#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct LiqPoolInitializeData {
    /// 流动性池目标流动性规模（单位：lamports），用于控制 LP 铸造速率
    pub lp_liquidity_target: u64,

    /// LP 赎回操作的最大手续费（以 Fee 表示）
    pub lp_max_fee: Fee,

    /// LP 赎回操作的最小手续费（以 Fee 表示）
    pub lp_min_fee: Fee,

    /// 协议抽取的 LP 国库收益分成比例
    pub lp_treasury_cut: Fee,
}


#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    /// 管理合约全局权限的管理员地址
    pub admin_authority: Pubkey,

    /// 管理验证者列表和 stake 分配的权限地址
    pub validator_manager_authority: Pubkey,

    /// 质押所需的最小 SOL 数量，防止创建垃圾账户
    pub min_stake: u64,

    /// 质押奖励的抽成比例（用于平台收益）
    pub rewards_fee: Fee,

    /// 初始化时的流动性池相关参数
    pub liq_pool: LiqPoolInitializeData,

    /// 为 stake_list 预留的额外记录空间
    pub additional_stake_record_space: u32,

    /// 为 validator_list 预留的额外记录空间
    pub additional_validator_record_space: u32,

    /// 用于 stake delta 调整计算的 slot 周期
    pub slots_for_stake_delta: u64,

    /// 可以暂停操作的权限地址（用于紧急控制）
    pub pause_authority: Pubkey,
}


impl<'info> Initialize<'info> {
    pub fn stake_pool(&self) -> &StakePoolConfig {
        &self.stake_pool_config
    }

    pub fn stake_pool_address(&self) -> &Pubkey {
        self.stake_pool_config.to_account_info().key
    }

    // 将 mint 账户调整为不能冻结
    fn set_authority<'a>(
        token_program: &AccountInfo<'a>,
        mint_authority: &AccountInfo<'a>,
        mint: &AccountInfo<'a>,
        seeds: &[&[u8]]
    ) -> Result<()> {
        token::set_authority(
            CpiContext::new_with_signer(
                token_program.clone(), 
                SetAuthority {
                    current_authority: mint_authority.clone(),
                    account_or_mint: mint.clone()
                }, 
                &[seeds]
            ), 
            AuthorityType::FreezeAccount, 
        None
        )
    }

    // 给初始化的 sol 系统账户转入lamport，避免账户被恶意占用
    fn pin_address_with_lamport<'a>(
        from: &AccountInfo<'a>,
        to: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        num: u64
    ) -> Result<()> {
        if **to.lamports.borrow() == 0 {
            invoke(
                &system_instruction::transfer(from.key, to.key, num), 
                &[from.clone(), to.clone(), system_program.clone()]
            )?;
        }

        Ok(())
    }

    fn init_lip_pool(
        &self, 
        data: LiqPoolInitializeData,
        lp_mint_authority_bump_seed: u8,
        sol_leg_bump_seed: u8,
        msol_leg_authority_bump_seed: u8,
    ) -> Result<LiqPool> {
        let liq_pool = LiqPool {
            lp_mint: self.lp_mint.key(),
            lp_mint_authority_bump_seed,
            sol_leg_bump_seed,
            msol_leg_authority_bump_seed,
            msol_leg: self.msol_leg.key(),
            lp_liquidity_target: data.lp_liquidity_target,
            lp_max_fee: data.lp_max_fee,
            lp_min_fee: data.lp_min_fee,
            treasury_cut: data.lp_treasury_cut,
            lp_supply: 0,
            lent_from_sol_leg: 0,
            liquidity_sol_cap: std::u64::MAX
        };

        liq_pool.validate()?;

        Ok(liq_pool)
    }

    pub fn process(
        &mut self, 
        initialize_data: InitializeData, 
        bumps: InitializeBumps
    ) -> Result<()> {
        require_lte!(
            initialize_data.rewards_fee, 
            StakePoolConfig::MAX_REWARD_FEE,
            StakingError::RewardsFeeIsTooHigh
        );
        require_keys_neq!(self.stake_pool_config.key(), self.stake_list.key());
        require_keys_neq!(self.stake_pool_config.key(), self.validator_list.key());
        require_keys_neq!(self.stake_list.key(), self.validator_list.key());
        require_gte!(
            initialize_data.slots_for_stake_delta,
            StakeSystem::MIN_UPDATE_WINDOW,
            StakingError::UpdateWindowIsTooLow
        );
        require_gte!(
            initialize_data.min_stake,
            StakePoolConfig::MIN_STAKE_LOWER_LIMIT,
            StakingError::MinStakeIsTooLow
        );

        let rent_exempt_for_token_acc = Rent::get()?.minimum_balance(spl_token::state::Account::LEN);

        // 给reserve_pda转入免租金
        Self::pin_address_with_lamport(
            &self.payer.to_account_info(), 
            &self.reserve_pda.to_account_info(), 
            &self.system_program.to_account_info(), 
            rent_exempt_for_token_acc
        )?;

        // 给sol_leg_pda转入免租金
        Self::pin_address_with_lamport(
            &self.payer.to_account_info(), 
            &self.sol_leg_pda.to_account_info(), 
            &self.system_program.to_account_info(), 
            rent_exempt_for_token_acc
        )?;

        // 将 msol_mint 和 lp_mint 设置为不可冻结
        let auth_key = self.stake_pool_config.key();
        let msol_auth_seeds = &[
            auth_key.as_ref(),
            StakePoolConfig::MSOL_MINT_AUTHORITY_SEED,
            &[bumps.msol_mint_authority]
        ];
        let lp_auth_seeds = &[
            auth_key.as_ref(),
            LiqPool::LP_MINT_AUTHORITY_SEED,
            &[bumps.lp_mint_authority]
        ];

        Self::set_authority(
            &self.token_program.to_account_info(), 
            &self.msol_mint_authority.to_account_info(), 
            &self.msol_mint.to_account_info(), 
            msol_auth_seeds
        )?;

        Self::set_authority(
            &self.token_program.to_account_info(), 
            &self.lp_mint_authority.to_account_info(), 
            &self.lp_mint.to_account_info(), 
            lp_auth_seeds
        )?;

        self.stake_pool_config.set_inner(StakePoolConfig {
            msol_mint: self.msol_mint.key(),
            admin_authority: initialize_data.admin_authority,
            operational_sol_account: self.operational_sol_account.key(),
            treasury_msol_account: self.treasury_msol_account.key(),
            reserve_bump_seed: bumps.reserve_pda,
            msol_mint_authority_bump_seed: bumps.msol_mint_authority,
            rent_exempt_for_token_acc,
            reward_fee: initialize_data.rewards_fee,
            stake_system: StakeSystem::new(
                self.stake_pool_address(), 
                self.stake_list.key(), 
                &mut self.stake_list.data.borrow_mut(), 
                initialize_data.slots_for_stake_delta, 
                initialize_data.min_stake, 
                0, 
                initialize_data.additional_stake_record_space
            )?,
            validator_system: ValidatorSystem::new(
                self.validator_list.key(), 
                &mut self.validator_list.data.borrow_mut(), 
                initialize_data.validator_manager_authority, 
                initialize_data.additional_validator_record_space
            )?,
            liq_pool: Self::init_lip_pool(
                self, 
                initialize_data.liq_pool, 
                bumps.lp_mint_authority,
                bumps.sol_leg_pda,
                bumps.msol_leg_authority
            )?,
            available_reserve_balance: 0,
            msol_supply: 0,
            msol_price: StakePoolConfig::PRICE_DENOMINATOR,
            min_deposit: 1,
            min_withdraw: 1,
            staking_sol_cap: std::u64::MAX,
            pause_authority: initialize_data.pause_authority,
            paused: false,
            last_stake_move_epoch: 0,
            stake_moved: 0,
            max_stake_moved_per_epoch: Fee::from_basis_points(10000), // 100%
        });
        Ok(())
    }

}

