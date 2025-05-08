use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL};

use crate::{calc::proportional, error::StakingError, require_lte, ID};

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


impl LiqPool {
    /// 用于派生 LP Mint 权限 PDA 的种子
    pub const LP_MINT_AUTHORITY_SEED: &'static [u8] = b"liq_mint";
    /// LP Mint 种子
    pub const LP_MINT_SEED: &'static [u8] = b"lp_mint";
    /// 用于派生 sol_leg PDA 的种子
    pub const SOL_LEG_SEED: &'static [u8] = b"liq_sol";
    /// 用于派生 mSOL leg 权限 PDA 的种子
    pub const MSOL_LEG_AUTHORITY_SEED: &'static [u8] = b"liq_st_sol_authority";
    /// 用于派生 mSOL leg 账户 PDA 的种子（作为字符串）
    pub const MSOL_LEG_SEED: &'static [u8] = b"liq_st_sol";
    /// 协议允许设定的最大 LP 赎回手续费（上限为 10%）
    pub const MAX_FEE: Fee = Fee::from_basis_points(1000); // 10%
    /// 流动性池最小目标 SOL 存量（50 SOL），用于控制 LP 铸造速率
    pub const MIN_LIQUIDITY_TARGET: u64 = 50 * LAMPORTS_PER_SOL; // 50 SOL
    /// 协议国库收益分成的最大比例（上限为 75%）
    pub const MAX_TREASURY_CUT: Fee = Fee::from_basis_points(7500);


    pub fn find_lp_mint_authority(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::LP_MINT_AUTHORITY_SEED], 
            &ID
        )
    }

    pub fn find_sol_leg_address(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::SOL_LEG_SEED], 
            &ID
        )
    }

    pub fn find_msol_leg_authority(stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], Self::MSOL_LEG_AUTHORITY_SEED],
            &ID,
        )
    }

    pub fn validate(&self) -> Result<()> {
        self.lp_min_fee
            .check()
            .map_err(|e| e.with_source(source!()))?;
        self.lp_max_fee
            .check()
            .map_err(|e| e.with_source(source!()))?;
        self.treasury_cut
            .check()
            .map_err(|e| e.with_source(source!()))?;

        require_lte!(
            self.lp_max_fee,
            Self::MAX_FEE,
            StakingError::LpMaxFeeIsTooHigh
        );
        require_gte!(
            self.lp_max_fee,
            self.lp_min_fee,
            StakingError::LpFeesAreWrongWayRound
        );
        require_gte!(
            self.lp_liquidity_target,
            Self::MIN_LIQUIDITY_TARGET,
            StakingError::LiquidityTargetTooLow
        );
        require_lte!(
            self.treasury_cut,
            Self::MAX_TREASURY_CUT,
            StakingError::TreasuryCutIsTooHigh
        );

        Ok(())
    }

    pub fn delta(&self) -> u32 {
        self.lp_max_fee.basis_points.saturating_sub(self.lp_min_fee.basis_points)
    }

    pub fn linear_fee(&self, lamports: u64) -> Fee {
        if lamports >= self.lp_liquidity_target {
            self.lp_min_fee
        } else {
            Fee {
                basis_points: self.lp_max_fee.basis_points - proportional(
                    self.delta() as u64, 
                    lamports, 
                    self.lp_liquidity_target
                ).unwrap() as u32
            }
        }
    }

    pub fn check_liquidity_cap(
        &self, 
        transfering_lamports: u64,
        sol_leg_balance: u64
    ) -> Result<()> {
        let result_amount = sol_leg_balance
            .checked_add(transfering_lamports)
            .ok_or(StakingError::MathOverflow)?;

        require_lte!(
            result_amount,
            self.liquidity_sol_cap,
            StakingError::LiquidityIsCapped
        );

        Ok(())
    }

    pub fn on_lp_mint(&mut self, amount: u64) {
        self.lp_supply += amount
    }

    pub fn on_lp_burn(&mut self, amount: u64) {
        self.lp_supply -= amount
    }
}
