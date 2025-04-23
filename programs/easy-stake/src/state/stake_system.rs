use anchor_lang::prelude::*;

use super::list::List;


#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSystem {
    /// 作为 stake account 列表的头部信息，记录每个项的大小和当前数量
    /// 实际的 stake account 数据存储在本账户后续字节中，由程序通过偏移访问
    pub stake_list: List,

    /// 正在延迟解押状态中的 SOL 总数（冷却中）
    pub delayed_unstake_cooling_down: u64,

    /// 派生 stake 存款地址时使用的 bump seed
    pub stake_deposit_bump_seed: u8,

    /// 派生 stake 提取地址时使用的 bump seed
    pub stake_withdraw_bump_seed: u8,

    /// 本 epoch 中 stake delta 的累计 slot 数，用于判断是否需要重新 stake
    pub solts_for_stake_delta: u64,

    /// 上一次进行 stake delta 计算的 epoch
    pub last_stake_delta_epoch: u64,

    /// 当前本合约已实际 stake 的 SOL 数量
    pub mint_stake: u64,
    
    /// 本 epoch 内额外进行的 stake delta 调整次数（限频使用）
    pub extra_stake_delta_runs: u32
}