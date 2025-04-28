use anchor_lang::prelude::*;

use crate::{error::StakingError, ID};

use super::list::List;


#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ValidatorRecord {
    /// 验证者的主账户地址（vote account），用于标识唯一的验证者节点
    pub validator_account: Pubkey,

    /// 当前该验证者已分配的质押 SOL 数量（单位：lamports），用于记录 stake 余额
    pub active_balance: u64,

    /// 验证者的得分，根据此分数按比例分配 stake
    pub score: u32,

    /// 上次 stake delta 更新所在的 epoch，用于防止在同一 epoch 内重复调整 stake
    pub last_stake_delta_epoch: u64,

    /// 与验证者地址和状态地址一起派生去重标志 PDA 的 bump 值，用于 PDA 派生和校验
    pub duplication_flag_bump_seed: u8,
}


impl ValidatorRecord {
    pub const DUPLICATE_FLAG_SEED: &'static [u8] = b"unique_validator";

    pub fn find_duplication_flag(state: &Pubkey, validator_account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &state.to_bytes()[..32],
                Self::DUPLICATE_FLAG_SEED,
                &validator_account.to_bytes()[..32],
            ],
            &ID,
        )
    }

    pub fn new(
        validator_account: Pubkey,
        score: u32,
        stake_pool: &Pubkey,
        duplication_flag_address: &Pubkey
    ) -> Result<Self> {
        let (actual_duplication_flag, duplication_flag_bump_seed) = Self::find_duplication_flag(
            stake_pool, 
            &validator_account
        );

        require_keys_eq!(
            actual_duplication_flag,
            *duplication_flag_address,
            StakingError::WrongValidatorDuplicationFlag
        );

        Ok(Self {
            validator_account,
            active_balance: 0,
            score,
            last_stake_delta_epoch: std::u64::MAX,
            duplication_flag_bump_seed
        })
    }
}



#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ValidatorList {}


impl Discriminator for ValidatorList {
    const DISCRIMINATOR: &'static [u8] = b"validatr";
}


impl AccountDeserialize for ValidatorList {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        *buf = &buf[8..];
        Ok(Self {})
    }

    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return err!(StakingError::InvalidValidatorListDiscriminator);
        }
        if buf[0..8] != *Self::DISCRIMINATOR {
            return err!(StakingError::InvalidValidatorListDiscriminator);
        }
        *buf = &buf[8..];
        Ok(Self {})
    }
}


impl AccountSerialize for ValidatorList {}


impl Owner for ValidatorList {
    fn owner() -> Pubkey {
        crate::ID
    }
}


#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ValidatorSystem {
    /// 管理所有验证者条目的列表头，实际数据存在账户后续区域
    pub validator_list: List,

    /// 拥有管理验证者权限的账户地址
    pub manager_authority: Pubkey,

    /// 所有验证者的总评分，用于 stake 分配计算
    pub total_validator_score: u32,

    /// 当前已分配给验证者的总质押 SOL 数量（单位：lamports）
    pub total_active_balance: u64,
}


impl ValidatorSystem {
    pub fn new(
        validator_list_account: Pubkey,
        validator_list_data: &mut [u8],
        manager_authority: Pubkey,
        additional_record_space: u32
    ) -> Result<Self> {
        Ok(Self {
            validator_list: List::new(
                ValidatorList::DISCRIMINATOR, 
                ValidatorRecord::default().try_to_vec().unwrap().len() as u32 + additional_record_space, 
                validator_list_account, 
                validator_list_data
            ).map_err(|e| e.with_account_name("validator_list"))?,
            manager_authority,
            total_validator_score: 0,
            total_active_balance: 0
        })
    }
}
