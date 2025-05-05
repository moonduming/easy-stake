//!  解质押

use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::token::{
    Mint, 
    Token, 
    TokenAccount, 
    transfer as transfer_token,
    Transfer as TransferToken
};

use crate::{error::StakingError, state::{Fee, LiqPool, StakePoolConfig}};


#[event]
pub struct LiquidUnstakeEvent {
    pub state: Pubkey,
    pub msol_owner: Pubkey,
    pub liq_pool_sol_balance: u64,
    pub liq_pool_msol_balance: u64,
    pub treasury_msol_balance: u64,
    pub user_msol_balance: u64,
    pub user_sol_balance: u64,
    pub msol_amount: u64,
    pub msol_fee: u64,
    pub treasury_msol_cut: u64,
    pub sol_amount: u64,
    // params used
    pub lp_liquidity_target: u64,
    pub lp_max_fee: Fee,
    pub lp_min_fee: Fee,
    pub treasury_cut: Fee,
}


#[derive(Accounts)]
pub struct Unstake<'info> {
    pub get_msol_from_authority: Signer<'info>,

    #[account(
        mut,
        has_one = treasury_msol_account,
        has_one = msol_mint
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        mint::authority = stake_pool_config
    )]
    pub msol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = stake_pool_config.treasury_msol_account,
        token::mint = msol_mint,
        token::authority = stake_pool_config
    )]
    pub treasury_msol_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            stake_pool_config.key().as_ref(),
            LiqPool::SOL_LEG_SEED
        ],
        bump = stake_pool_config.liq_pool.sol_leg_bump_seed
    )]
    pub liq_pool_sol_leg_pda: SystemAccount<'info>,

    #[account(
        mut,
        address = stake_pool_config.liq_pool.msol_leg
    )]
    pub liq_pool_msol_leg: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = msol_mint,
        token::authority = get_msol_from_authority
    )]
    pub get_msol_from: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub transfer_sol_to: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>

}


impl<'info> Unstake<'info> {
    pub fn process(&mut self, msol_amount: u64) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        let user_sol_balance = self.transfer_sol_to.lamports();
        let user_msol_balance = self.get_msol_from.amount;
        let treasury_msol_balance = self.treasury_msol_account.amount;
        let liq_pool_msol_balance = self.liq_pool_msol_leg.amount;
        let liq_pool_sol_balance = self.liq_pool_sol_leg_pda.lamports();
        let liq_pool_available_sol_balance = liq_pool_sol_balance.saturating_sub(
                self.stake_pool_config.rent_exempt_for_token_acc
            );
        
        // 计算能兑换到的 sol
        let user_remove_lamports = self.stake_pool_config.msol_to_sol(msol_amount)?;
        // 计算兑换手续费
        let liquid_unstake_fee = if user_remove_lamports >= liq_pool_available_sol_balance {
            // 兑换的 sol 数量超过池子sol总量，直接收取最大手续费
            self.stake_pool_config.liq_pool.lp_max_fee
        } else {
            // 用户提取后池子剩余的 SOL，后续根据它计算手续费。
            // 池子越接近枯竭，手续费越高（linear_fee 根据剩余量计算）。
            let after_lamports = liq_pool_available_sol_balance - user_remove_lamports;
            self.stake_pool_config.liq_pool.linear_fee(after_lamports)
        };

        let msol_fee = liquid_unstake_fee.apply(msol_amount);
        msg!("msol_fee {}", msol_fee);

        // 扣除手续费后能提取到的 sol
        let working_lamports_value = self.stake_pool_config.msol_to_sol(msol_amount - msol_fee)?;

        require_gte!(
            working_lamports_value,
            self.stake_pool_config.min_withdraw,
            StakingError::WithdrawAmountIsTooLow
        );

        // 判断提取数量是否超过池子 sol 总量
        if working_lamports_value + self.stake_pool_config.rent_exempt_for_token_acc 
            > self.liq_pool_sol_leg_pda.lamports() 
        {
            return err!(StakingError::InsufficientLiquidity);
        }

        // 转帐 sol
        if working_lamports_value > 0 {
            transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(), 
                    Transfer {
                        from: self.liq_pool_sol_leg_pda.to_account_info(),
                        to: self.transfer_sol_to.to_account_info()
                    }, 
                    &[&[
                        self.stake_pool_config.key().as_ref(),
                        LiqPool::SOL_LEG_SEED,
                        &[self.stake_pool_config.liq_pool.sol_leg_bump_seed]
                    ]],
                ), 
                working_lamports_value
            )?;
        }

        let treasury_msol_cut = self.stake_pool_config.liq_pool.treasury_cut.apply(msol_fee);
        msg!("treasury_msol_cut {}", treasury_msol_cut);

        // 扣除国库手续费后的 msol 入池
        transfer_token(
            CpiContext::new(
                self.token_program.to_account_info(), 
                TransferToken {
                    from: self.get_msol_from.to_account_info(),
                    to: self.liq_pool_msol_leg.to_account_info(),
                    authority: self.get_msol_from_authority.to_account_info()
                }
            ), 
        msol_amount - treasury_msol_cut
        )?;

        // 将手续费转给 国库
        if treasury_msol_cut > 0 {
            transfer_token(
                CpiContext::new(
                    self.token_program.to_account_info(), 
                    TransferToken {
                        from: self.get_msol_from.to_account_info(),
                        to: self.treasury_msol_account.to_account_info(),
                        authority: self.get_msol_from_authority.to_account_info()
                    }
                ), 
                treasury_msol_cut
            )?;
        }
        
        emit!(LiquidUnstakeEvent {
            state: self.stake_pool_config.key(),
            msol_owner: self.get_msol_from.owner,
            msol_amount,
            liq_pool_sol_balance,
            liq_pool_msol_balance,
            treasury_msol_balance,
            user_msol_balance,
            user_sol_balance,
            msol_fee,
            treasury_msol_cut,
            sol_amount: working_lamports_value,
            lp_liquidity_target: self.stake_pool_config.liq_pool.lp_liquidity_target,
            lp_max_fee: self.stake_pool_config.liq_pool.lp_max_fee,
            lp_min_fee: self.stake_pool_config.liq_pool.lp_min_fee,
            treasury_cut: self.stake_pool_config.liq_pool.treasury_cut
        });

        Ok(())
    }
}
