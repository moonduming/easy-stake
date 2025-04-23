use anchor_lang::prelude::*;
use borsh::BorshSchema;


#[derive(Default, Clone, AnchorSerialize, AnchorDeserialize, BorshSchema, Debug)]
pub struct List {
    /// 公共账户地址，通常表示该列表属于哪个账户或PDA
    pub account: Pubkey,
    /// 每个列表项的字节大小，用于解析或遍历列表项
    pub item_size: u32,
    /// 当前列表中的项数
    pub count: u32,
    /// 预留字段，保留未来扩展使用
    pub _reserved1: Pubkey,
    /// 预留字段，保留未来扩展使用
    pub _reserved2: Pubkey
}