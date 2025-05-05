//! 质押用户系统信息

use anchor_lang::{prelude::*, solana_program::clock::Epoch};

use super::list::List;
use crate::{error::StakingError, ID};

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct StakeRecord {
    /// 用户质押时创建的 stake account 地址
    pub stake_account: Pubkey,

    /// 上次更新时记录的已委托 lamports（用于计算 delta）
    pub last_update_delegated_lamports: u64,

    /// 上次更新时的 epoch（用于判断是否需要更新）
    pub last_update_epoch: u64,

    /// 是否是紧急解押状态（1 表示冷却中，0 表示正常）
    /// 1 表示紧急解押后处于冷却中，0 表示正常状态
    pub is_emergency_unstaking: u8,
}

impl StakeRecord {
    pub fn new(
        stake_account: &Pubkey,
        delegated_lamports: u64,
        clock: &Clock,
        is_emergency_unstaking: u8,
    ) -> Self {
        Self {
            stake_account: *stake_account,
            last_update_delegated_lamports: delegated_lamports,
            last_update_epoch: clock.epoch,
            is_emergency_unstaking,
        }
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct StakeList {}

impl Discriminator for StakeList {
    const DISCRIMINATOR: &'static [u8] = b"staker__";
}

impl AccountDeserialize for StakeList {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        *buf = &buf[8..];
        Ok(Self {})
    }

    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return err!(StakingError::InvalidStakeListDiscriminator);
        }
        if buf[0..8] != *Self::DISCRIMINATOR {
            return err!(StakingError::InvalidStakeListDiscriminator);
        }

        *buf = &buf[8..];
        Ok(Self {})
    }
}

impl AccountSerialize for StakeList {}

impl Owner for StakeList {
    fn owner() -> Pubkey {
        ID
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSystem {
    /// 作为 stake account 列表的头部信息，记录每个项的大小和当前数量
    /// 实际的 stake account 数据存储在本账户后续字节中，由程序通过偏移访问
    pub stake_list: List,

    /// 派生 stake 存款地址时使用的 bump seed
    pub stake_deposit_bump_seed: u8,

    /// 派生 stake 提取地址时使用的 bump seed
    pub stake_withdraw_bump_seed: u8,

    /// 本 epoch 中 stake delta 的累计 slot 数，用于判断是否需要重新 stake
    pub slots_for_stake_delta: u64,

    /// 上一次进行 stake delta 计算的 epoch
    pub last_stake_delta_epoch: u64,

    /// 当前本合约已实际 stake 的 SOL 数量
    pub min_stake: u64,

    /// 本 epoch 内额外进行的 stake delta 调整次数（限频使用）
    pub extra_stake_delta_runs: u32,
}

impl StakeSystem {
    /// 用于派生提取 stake 账户地址的种子
    pub const STAKE_WITHDRAW_SEED: &'static [u8] = b"withdraw";
    /// 用于派生存入 stake 账户地址的种子
    pub const STAKE_DEPOSIT_SEED: &'static [u8] = b"deposit";
    /// stake delta 重新计算前必须经过的最小 slot 间隔
    pub const MIN_UPDATE_WINDOW: u64 = 3_000;
    /// StakeRecord 长度
    pub const STAKE_RECORD_LEN: usize = 49;

    pub fn new(
        stake_pool: &Pubkey,
        stake_list_account: Pubkey,
        stake_list_data: &mut [u8],
        slots_for_stake_delta: u64,
        min_stake: u64,
        extra_stake_delta_runs: u32,
        additional_stake_record_space: u32,
    ) -> Result<Self> {
        let stake_list = List::new(
            StakeList::DISCRIMINATOR,
            Self::STAKE_RECORD_LEN as u32 + additional_stake_record_space,
            stake_list_account,
            stake_list_data,
        )
        .map_err(|e| e.with_account_name("stake_list"))?;

        Ok(Self { 
            stake_list, 
            stake_deposit_bump_seed: Self::find_stake_deposit_authority(stake_pool).1, 
            stake_withdraw_bump_seed: Self::find_stake_withdraw_authority(stake_pool).1, 
            slots_for_stake_delta, 
            last_stake_delta_epoch: Epoch::MAX, 
            min_stake, 
            extra_stake_delta_runs
        })
    }

    pub fn find_stake_deposit_authority(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::STAKE_DEPOSIT_SEED],
            &ID,
        )
    }

    pub fn find_stake_withdraw_authority(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::STAKE_WITHDRAW_SEED],
            &ID,
        )
    }
}
