use anchor_lang::prelude::*;

use super::list::List;


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