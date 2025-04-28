//! 账户配置文件
use std::mem::MaybeUninit;

use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL};

pub mod fee;
pub mod stake_system;
pub mod validator_system;
pub mod list;
pub mod liq_pool;

pub use fee::Fee;
pub use stake_system::StakeSystem;
pub use validator_system::ValidatorSystem;
pub use liq_pool::LiqPool;

use crate::ID;


#[account]
pub struct StakePoolConfig {
    /// mSOL 的 mint 地址，用于铸造和销毁 mSOL
    pub msol_mint: Pubkey,

    /// 管理员地址，拥有修改参数权限
    pub admin_authority: Pubkey,

    /// 用于操作系统 SOL 支付的账户地址
    pub operational_sol_account: Pubkey,

    /// 接收手续费的 mSOL 国库账户
    pub treasury_msol_account: Pubkey,

    /// reserve_pda 的 bump，用于 PDA 派生签名
    pub reserve_bump_seed: u8,

    /// mSOL mint authority 的 bump，用于 PDA 派生签名
    pub msol_mint_authority_bump_seed: u8,

    /// 创建 token account 时所需的最低 rent 保留金
    pub rent_exempt_for_token_acc: u64,

    /// mSOL staking 收益的协议抽成，单位为分数（如 500 = 0.5%
    pub reward_fee: Fee,

    /// 质押账户系统信息
    pub stake_system: StakeSystem,

    /// 验证者管理系统信息
    pub validator_system: ValidatorSystem,

    /// 流动性池账户地址
    pub liq_pool: LiqPool,

    /// reserve_pda 中可用于 stake 的 SOL 数量
    pub available_reserve_balance: u64,

    /// 当前 mSOL 的总供应量
    pub msol_supply: u64,

    /// 当前 1 mSOL 对应的 SOL 价值（仅用于显示）
    pub msol_price: u64,

    /// 用户最小存入 SOL 限额
    pub min_deposit: u64,

    /// 用户最小赎回 SOL 限额
    pub min_withdraw: u64,

    /// staking 的总上限，超过则拒绝新增 stake
    pub staking_sol_cap: u64,

    /// 暂停权限地址，可用于应急情况禁用指令
    pub pause_authority: Pubkey,

    /// 是否处于暂停状态（true 表示暂停中）
    pub paused: bool,

    /// 上一次进行 stake 调整的 epoch 编号
    pub last_stake_move_epoch: u64,

    /// 本 epoch 中已移动的 stake 数量
    pub stake_moved: u64,

    /// 每个 epoch 允许移动的最大 stake 数量
    pub max_stake_moved_per_epoch: Fee,
}


impl StakePoolConfig {
    /// mSOL 价格的分母，用于计算价格比例
    pub const PRICE_DENOMINATOR: u64 = 0x1_0000_0000;
    /// 全局账户种子
    pub const STAKE_POOL_CONFIG_SEED: &'static [u8] = b"stake_pool";
    /// reserve PDA 派生种子，用于生成保留账户地址
    pub const RESERVE_SEED: &'static [u8] = b"reserve";
    /// mSOL mint authority PDA 派生种子，用于 mint 权限管理
    pub const MSOL_MINT_AUTHORITY_SEED: &'static [u8] = b"st_mint";
    /// msom mint 种子
    pub const MSOL_MINT_SEED: &'static [u8] = b"msol_mint";
    /// 质押列表 PDA 派生种子字符串
    pub const STAKE_LIST_SEED: &'static str = "stake_list";
    /// 验证者列表 PDA 派生种子字符串
    pub const VALIDATOR_LIST_SEED: &'static str = "validator_list";
    /// 用于托管mSOL的Token PDA账户 种子
    pub const TREASURY_MSOL_SEED: &'static [u8] = b"treasury_msol";
    /// 最大奖励手续费，单位为基点（1000 = 10%）
    pub const MAX_REWARD_FEE: Fee = Fee::from_basis_points(1_000);
    /// 最大单笔提现金额，单位为 lamports（0.1 SOL）
    pub const MAX_WITHDRAW_ATOM: u64 = LAMPORTS_PER_SOL / 10;
    /// 最小质押下限，单位为 lamports（0.01 SOL）
    pub const MIN_STAKE_LOWER_LIMIT: u64 = LAMPORTS_PER_SOL / 100;


    /// 获取 StakePoolConfig 结构体在链上账户中所需的总存储空间（单位：字节）。
    /// 包括序列化后的长度和 Anchor discriminator（8 字节）。
    /// 用于创建账户时设置 `space` 参数，确保分配足够空间。
    pub fn serialized_len() -> usize {
        unsafe { MaybeUninit::<Self>::zeroed().assume_init() }
        .try_to_vec()
        .unwrap().len() 
        + 8
    }

    pub fn find_msol_mint_authority(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::MSOL_MINT_AUTHORITY_SEED],
            &ID
        )
    }
}
