use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::token::{
    burn, transfer as transfer_token, Burn, Mint, Token, TokenAccount, Transfer as TransferToken
};

use crate::{
    calc::proportional, 
    error::StakingError, 
    require_lte, 
    state::{LiqPool, StakePoolConfig}
};


#[event]
pub struct RemoveLiquidityEvent {
    pub state: Pubkey,
    pub sol_leg_balance: u64,
    pub msol_leg_balance: u64,
    pub user_lp_balance: u64,
    pub user_sol_balance: u64,
    pub user_msol_balance: u64,
    pub lp_mint_supply: u64,
    pub lp_burned: u64,
    pub sol_out_amount: u64,
    pub msol_out_amount: u64,
}


#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    pub burn_from_authority: Signer<'info>,

    #[account(
        mut,
        token::mint = lp_mint,
        token::authority = burn_from_authority
    )]
    pub burn_from: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        address = stake_pool_config.liq_pool.lp_mint
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub transfer_sol_to: SystemAccount<'info>,

    #[account(
        mut,
        token::mint = stake_pool_config.msol_mint
    )]
    pub transfer_msol_to: Box<Account<'info, TokenAccount>>,

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
        address = stake_pool_config.liq_pool.msol_leg,
        token::authority = liq_pool_msol_leg_authority
    )]
    pub liq_pool_msol_leg: Box<Account<'info, TokenAccount>>,

    /// CHECK: PDA
    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            LiqPool::MSOL_LEG_AUTHORITY_SEED
        ],
        bump = stake_pool_config.liq_pool.msol_leg_authority_bump_seed
    )]
    pub liq_pool_msol_leg_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}


impl<'info> RemoveLiquidity<'info> {
    pub fn process(&mut self, tokens: u64) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        require_lte!(
            tokens,
            self.burn_from.amount,
            StakingError::NotEnoughUserFunds
        );

        let user_lp_balance = self.burn_from.amount;
        let user_sol_balance = self.transfer_sol_to.lamports();
        let user_msol_balance = self.transfer_msol_to.amount;
        let sol_leg_balance = self.liq_pool_sol_leg_pda.lamports();
        let msol_leg_balance = self.liq_pool_msol_leg.amount;

        let lp_mint_supply = self.lp_mint.supply;
        if lp_mint_supply > self.stake_pool_config.liq_pool.lp_supply {
            msg!("有人未经我们的许可或发现漏洞而铸造了 LP 代币");
            return Err(StakingError::UnauthorizedOrExploitedLPMinting.into());
        } else {
            self.stake_pool_config.liq_pool.lp_supply = lp_mint_supply;
        }
        msg!("mSOL-SOL-LP total supply {}", lp_mint_supply);

        let sol_out_amount = proportional(
            tokens, 
            sol_leg_balance - self.stake_pool_config.rent_exempt_for_token_acc, 
            self.stake_pool_config.liq_pool.lp_supply
        )?;
        let msol_out_amount = proportional(
            tokens, 
            msol_leg_balance, 
            self.stake_pool_config.liq_pool.lp_supply
        )?;

        require_gte!(
            sol_out_amount + self.stake_pool_config.msol_to_sol(msol_out_amount)?,
            self.stake_pool_config.min_withdraw,
            StakingError::WithdrawAmountIsTooLow
        );

        msg!(
            "SOL out amount:{}, mSOL out amount:{}",
            sol_out_amount,
            msol_out_amount
        );

        if sol_out_amount > 0 {
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
                    ]]
                ), 
                sol_out_amount
            )?;
        }

        if msol_out_amount > 0 {
            transfer_token(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(), 
                    TransferToken {
                        from: self.liq_pool_msol_leg.to_account_info(),
                        to: self.transfer_msol_to.to_account_info(),
                        authority: self.liq_pool_msol_leg_authority.to_account_info()
                    }, 
                    &[&[
                        self.stake_pool_config.key().as_ref(),
                        LiqPool::MSOL_LEG_AUTHORITY_SEED,
                        &[self.stake_pool_config.liq_pool.msol_leg_authority_bump_seed]
                    ]]
                ), 
                msol_out_amount
            )?;
        }

        // 销毁 lp token
        burn(
            CpiContext::new(
                self.token_program.to_account_info(), 
                Burn {
                    mint: self.lp_mint.to_account_info(),
                    from: self.burn_from.to_account_info(),
                    authority: self.burn_from_authority.to_account_info()
                }
            ), 
            tokens
        )?;

        self.stake_pool_config.liq_pool.on_lp_burn(tokens);

        emit!(RemoveLiquidityEvent {
            state: self.stake_pool_config.key(),
            sol_leg_balance,
            msol_leg_balance,
            user_lp_balance,
            user_sol_balance,
            user_msol_balance,
            lp_mint_supply,
            lp_burned: tokens,
            sol_out_amount,
            msol_out_amount,
        });

        Ok(())
    }
}
