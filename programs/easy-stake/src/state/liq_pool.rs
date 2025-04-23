use anchor_lang::prelude::*;

use super::Fee;


#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct LiqPool {
    /// LP 代币的 Mint 地址
    pub lp_mint: Pubkey,

    /// LP Mint 权限 PDA 的 bump seed，用于派生签名
    pub lp_mint_authority_bump_seed: u8,

    /// sol_leg 账户 PDA 的 bump seed
    pub sol_leg_bump_seed: u8,

    /// mSOL leg 授权 PDA 的 bump seed
    pub msol_leg_authority_bump_seed: u8,

    /// mSOL leg 账户地址（存储 mSOL 的账户）
    pub msol_leg: Pubkey,

    /// 希望流动性池达到的目标 LP 总量（用于控制增长）
    pub lp_liquidity_target: u64,

    /// LP 提供者赎回时收取的最大手续费
    pub lp_max_fee: Fee,

    /// LP 提供者赎回时收取的最小手续费
    pub lp_min_fee: Fee,

    /// 协议抽取的国库分成比例
    pub treasury_cut: Fee,

    /// 当前已发行的 LP token 总量
    pub lp_supply: u64,

    /// 当前从 sol_leg 中借出的 SOL 数量（用于流动性支持）
    pub lent_from_sol_leg: u64,

    /// sol_leg 最大可存储的 SOL 数量上限（防止过度注入）
    pub liquidity_sol_cap: u64,
}