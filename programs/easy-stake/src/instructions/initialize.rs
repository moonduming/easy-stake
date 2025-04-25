use anchor_lang::{prelude::*, solana_program::program_pack::Pack};
use anchor_spl::token::{spl_token, Mint, TokenAccount};


use crate::{checks::{check_freeze_authority, check_mint_authority, check_mint_empty, check_token_mint, check_token_owner}, error::StakingError, require_lte, state::{Fee, LiqPool, StakePoolConfig, StakeSystem, ValidatorSystem}};


#[derive(Accounts)]
pub struct Initialize<'info> {
    /// 初始化时的质押池配置账户，要求账户数据为零
    #[account(zero)]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    /// 质押池配置的储备PDA账户，使用种子和bump生成
    #[account(
        seeds = [
            &stake_pool_config.key().to_bytes(),
            StakePoolConfig::RESERVE_SEED
        ],
        bump
    )]
    pub reserve_pda: SystemAccount<'info>,

    /// CHECK: 初始化时允许未校验账户，后续会在逻辑中验证 owner 和结构合法性
    pub stake_list: UncheckedAccount<'info>,

    /// CHECK: 此账户在初始化后将由逻辑手动校验其 owner 和数据内容
    pub validator_list: UncheckedAccount<'info>,

    /// mSOL代币铸造账户
    pub msol_mint: Box<Account<'info, Mint>>,

    /// 操作用的SOL账户
    pub operational_sol_account: SystemAccount<'info>,

    /// 流动性池相关账户集合
    pub liq_pool: LiqPoolInitialize<'info>,

    /// 用于托管mSOL的Token账户，需与msol_mint对应
    #[account(token::mint = msol_mint)]
    pub treasury_msol_account: Box<Account<'info, TokenAccount>>,

}


#[derive(Accounts)]
pub struct LiqPoolInitialize<'info> {
    /// LP代币铸造账户
    pub lp_mint: Box<Account<'info, Mint>>,
    /// 用于流动性池中存储 SOL 的 PDA 账户
    pub sol_leg_pda: SystemAccount<'info>,
    /// 用于流动性池中存储 MSOL 的 PDA 账户
    pub msol_leg: Box<Account<'info, TokenAccount>>
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

    fn check_reserve_pda(&self, required_lamports: u64) -> Result<()> {
        require_eq!(self.reserve_pda.lamports(), required_lamports);

        Ok(())
    }

    fn check_msol_mint(&mut self) -> Result<u8> {
        let (authority_address, bump) = StakePoolConfig::find_msol_mint_authority(self.stake_pool_address());

        check_mint_authority(&self.msol_mint, &authority_address, "msol_mint")?;
        check_mint_empty(&self.msol_mint, "msol_mint")?;
        check_freeze_authority(&self.msol_mint, "msol_mint")?;

        Ok(bump)
    }

    pub fn process(
        &mut self, 
        initialize_data: InitializeData, 
        reserve_pda_bump: u8
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
        self.check_reserve_pda(rent_exempt_for_token_acc)?;

        let msol_mint_authority_bump_seed = self.check_msol_mint()?;

        self.stake_pool_config.set_inner(StakePoolConfig {
            msol_mint: self.msol_mint.key(),
            admin_authority: initialize_data.admin_authority,
            operational_sol_account: self.operational_sol_account.key(),
            treasury_msol_account: self.treasury_msol_account.key(),
            reserve_bump_seed: reserve_pda_bump,
            msol_mint_authority_bump_seed,
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
            liq_pool: LiqPoolInitialize::process(
                self, 
                initialize_data.liq_pool, 
                rent_exempt_for_token_acc
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


impl<'info> LiqPoolInitialize<'info> {
    pub fn check_lp_mint(parent: &Initialize) -> Result<u8> {
        require_keys_neq!(parent.liq_pool.lp_mint.key(), parent.msol_mint.key());
        let (authority_address, bump) = LiqPool::find_lp_mint_authority(parent.stake_pool_address());

        check_mint_authority(&parent.liq_pool.lp_mint, &authority_address, "lp_mint")?;
        check_mint_empty(&parent.liq_pool.lp_mint, "lp_mint")?;
        check_freeze_authority(&parent.liq_pool.lp_mint, "lp_mint")?;

        Ok(bump)
    }

    pub fn check_sol_leg(parent: &Initialize, required_lamports: u64) -> Result<u8> {
        let (address, bump) = LiqPool::find_sol_leg_address(parent.stake_pool_address());
        require_keys_eq!(parent.liq_pool.sol_leg_pda.key(), address);
        require_eq!(parent.liq_pool.sol_leg_pda.lamports(), required_lamports);

        Ok(bump)
    }

    pub fn check_msol_leg(parent: &Initialize) -> Result<u8> {
        check_token_mint(
            &parent.liq_pool.msol_leg, 
            &parent.msol_mint.key(), 
            "liq_msol"
        )?;
        let (msol_authority, bump) = LiqPool::find_msol_leg_authority(parent.stake_pool_address());
        check_token_owner(&parent.liq_pool.msol_leg, &msol_authority, "liq_msol_leg")?;

        Ok(bump)
    }

    pub fn process(
        parent: &Initialize, 
        data: LiqPoolInitializeData,
        required_sol_leg_lamports: u64
    ) -> Result<LiqPool> {
        let lp_mint_authority_bump_seed = Self::check_lp_mint(parent)?;
        let sol_leg_bump_seed = Self::check_sol_leg(parent, required_sol_leg_lamports)?;
        let msol_leg_authority_bump_seed = Self::check_msol_leg(parent)?;
        let liq_pool = LiqPool {
            lp_mint: parent.liq_pool.lp_mint.key(),
            lp_mint_authority_bump_seed,
            sol_leg_bump_seed,
            msol_leg_authority_bump_seed,
            msol_leg: parent.liq_pool.msol_leg.key(),
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
}
