//! 用户质押逻辑

use anchor_lang::{prelude::*, system_program::{self, transfer, Transfer}};
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        mint_to, transfer as transfer_tokens, Mint, MintTo, Token, TokenAccount, Transfer as TransferTokens
    }
};

use crate::{error::StakingError, require_lte, state::{LiqPool, StakePoolConfig}};


#[event]
pub struct DepositEvent {
    pub state: Pubkey,
    pub sol_owner: Pubkey,
    pub user_sol_balance: u64,
    pub user_msol_balance: u64,
    pub sol_leg_balance: u64,
    pub msol_leg_balance: u64,
    pub reserve_balance: u64,
    pub sol_swapped: u64,
    pub msol_swapped: u64,
    pub sol_deposited: u64,
    pub msol_minted: u64,
    pub total_virtual_staked_lamports: u64,
    pub msol_supply: u64
}


#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        owner = system_program::ID
    )]
    pub user: Signer<'info>,

    #[account(
        mut,
        has_one = msol_mint
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(mut)]
    pub msol_mint: Box<Account<'info, Mint>>,

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

    /// CHECK: PDA
    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            LiqPool::MSOL_LEG_AUTHORITY_SEED
        ],
        bump = stake_pool_config.liq_pool.msol_leg_authority_bump_seed
    )]
    pub liq_pool_msol_leg_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            stake_pool_config.key().as_ref(),
            StakePoolConfig::RESERVE_SEED
        ],
        bump = stake_pool_config.reserve_bump_seed
    )]
    pub reserve_pda: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = msol_mint,
        associated_token::authority = user
    )]
    pub mint_to: Box<Account<'info, TokenAccount>>,

    /// CHECK: PDA
    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            StakePoolConfig::MSOL_MINT_AUTHORITY_SEED
        ],
        bump = stake_pool_config.msol_mint_authority_bump_seed
    )]
    pub msol_mint_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>
}


impl<'info> Deposit<'info> {
    pub fn process(&mut self, lamports: u64) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        require_gte!(
            lamports, 
            self.stake_pool_config.min_deposit, 
            StakingError::DepositAmountIsTooLow
        );

        let user_sol_balance = self.user.lamports();
        require_gte!(
            user_sol_balance,
            lamports,
            StakingError::NotEnoughUserFunds
        );

        require_lte!(
            self.msol_mint.supply,
            self.stake_pool_config.msol_supply,
            StakingError::UnregisteredMsolMinted
        );

        let user_msol_balance = self.mint_to.amount;
        let reserve_balance = self.reserve_pda.lamports();
        let sol_leg_balance = self.liq_pool_sol_leg_pda.lamports();
        let total_virtual_staked_lamports = self.stake_pool_config.total_staked_lamports();
        let msol_supply = self.stake_pool_config.msol_supply;

        let user_msol_buy_order = self.stake_pool_config.calc_msol_from_lamports(lamports)?;
        msg!("--- user_MSOL_buy_order {}", user_msol_buy_order);

        let msol_leg_balance = self.liq_pool_msol_leg.amount;
        let msol_swapped = user_msol_buy_order.min(msol_leg_balance);
        msg!("--- swap_MSOL_max {}", msol_swapped);

        let sol_swapped = if msol_swapped > 0 {
            let sol_swapped = if user_msol_buy_order == msol_swapped {
                lamports
            } else {
                self.stake_pool_config.msol_to_sol(msol_swapped)?
            };

            // 给用户发放 mSOL
            transfer_tokens(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(), 
                    TransferTokens {
                        from: self.liq_pool_msol_leg.to_account_info(),
                        to: self.mint_to.to_account_info(),
                        authority: self.liq_pool_msol_leg_authority.to_account_info()
                    }, 
                    &[&[
                        &self.stake_pool_config.key().to_bytes(),
                        LiqPool::MSOL_LEG_AUTHORITY_SEED,
                        &[self.stake_pool_config.msol_mint_authority_bump_seed]
                    ]]
                ),
                msol_swapped
            )?;

            // 将用户的 sol 转入池子 sol 代币账户
            transfer(
                CpiContext::new(
                    self.system_program.to_account_info(), 
                    Transfer {
                        from: self.user.to_account_info(),
                        to: self.liq_pool_sol_leg_pda.to_account_info()
                    }
                ), 
                sol_swapped
            )?;

            sol_swapped
        } else {
            0
        };

        let sol_deposited = lamports - sol_swapped;
        if sol_deposited > 0 {
            self.stake_pool_config.check_staking_cap(sol_deposited)?;

            transfer(
                CpiContext::new(
                    self.system_program.to_account_info(), 
                Transfer {
                    from: self.user.to_account_info(),
                    to: self.reserve_pda.to_account_info()
                    }
                ), 
            sol_deposited
            )?;

            self.stake_pool_config.on_transfer_to_reserve(sol_deposited);
        }

        let msol_minted = user_msol_buy_order - msol_swapped;
        if msol_minted > 0 {
            msg!("--- msol_to_mint {}", msol_minted);
            mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(), 
                    MintTo {
                        mint: self.msol_mint.to_account_info(),
                        to: self.mint_to.to_account_info(),
                        authority: self.msol_mint_authority.to_account_info()
                    }, 
                    &[&[
                        &self.stake_pool_config.key().to_bytes(),
                        StakePoolConfig::MSOL_MINT_AUTHORITY_SEED,
                        &[self.stake_pool_config.msol_mint_authority_bump_seed]
                    ]]
                ), 
                msol_minted
            )?;

            self.stake_pool_config.on_msol_mint(msol_minted);
        }

        emit!(DepositEvent {
            state: self.stake_pool_config.key(),
            sol_owner: self.user.key(),
            user_sol_balance,
            user_msol_balance,
            sol_leg_balance,
            msol_leg_balance,
            reserve_balance,
            sol_swapped,
            msol_swapped,
            sol_deposited,
            msol_minted,
            total_virtual_staked_lamports,
            msol_supply
        });

        Ok(())
    }
}
