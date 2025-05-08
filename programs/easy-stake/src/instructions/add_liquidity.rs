//! 池子添加流动性

use anchor_lang::{
    prelude::*,  
    system_program::{ID as sys_id, transfer, Transfer}
};
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{ 
        mint_to, Mint, MintTo, Token, TokenAccount
    }
};

use crate::{calc::shares_from_value, error::StakingError, require_lte, state::{LiqPool, StakePoolConfig}};


#[event]
pub struct AddLiquidityEvent {
    pub state: Pubkey,
    pub sol_owner: Pubkey,
    pub user_sol_balance: u64,
    pub user_lp_balance: u64,
    pub sol_leg_balance: u64,
    pub lp_supply: u64,
    pub sol_added_amount: u64,
    pub lp_minted: u64,
    // MSOL price used
    pub total_virtual_staked_lamports: u64,
    pub msol_supply: u64,
}


#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        owner = sys_id
    )]
    pub transfer_from: Signer<'info>,

    #[account(
        mut,
        seeds = [StakePoolConfig::STAKE_POOL_CONFIG_SEED],
        bump = stake_pool_config.stake_bump
    )]
    pub stake_pool_config: Box<Account<'info, StakePoolConfig>>,

    #[account(
        mut,
        address = stake_pool_config.liq_pool.lp_mint,
        mint::authority = lp_mint_authority
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA
    #[account(
        seeds = [
            stake_pool_config.key().as_ref(),
            LiqPool::LP_MINT_AUTHORITY_SEED
        ],
        bump = stake_pool_config.liq_pool.lp_mint_authority_bump_seed
    )]
    pub lp_mint_authority: UncheckedAccount<'info>,

    #[account(address = stake_pool_config.liq_pool.msol_leg)]
    pub liq_pool_msol_leg: Box<Account<'info, TokenAccount>>,

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
        init_if_needed,
        payer = transfer_from,
        associated_token::mint = lp_mint,
        associated_token::authority = transfer_from
    )]
    pub mint_to: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

impl<'info> AddLiquidity<'info> {
    pub fn process(&mut self, lamports: u64) -> Result<()> {
        require!(!self.stake_pool_config.paused, StakingError::ProgramIsPaused);

        require_gte!(
            lamports,
            self.stake_pool_config.min_deposit,
            StakingError::DepositAmountIsTooLow
        );

        let user_sol_balance = self.transfer_from.lamports();
        require_lte!(
            lamports,
            user_sol_balance,
            StakingError::NotEnoughUserFunds
        );

        self.stake_pool_config.liq_pool.check_liquidity_cap(
            lamports, 
            self.liq_pool_sol_leg_pda.lamports()
        )?;

        require_lte!(
            self.lp_mint.supply,
            self.stake_pool_config.liq_pool.lp_supply,
            StakingError::UnregisteredLPMinted
        );

        self.stake_pool_config.liq_pool.lp_supply = self.lp_mint.supply;

        let total_virtual_staked_lamports = self.stake_pool_config.total_staked_lamports();
        let msol_supply = self.stake_pool_config.msol_supply;

        let sol_leg_balance = self.liq_pool_sol_leg_pda.lamports();
        let sol_leg_available_balance = sol_leg_balance - self.stake_pool_config.rent_exempt_for_token_acc;
        let msol_leg_value = self.stake_pool_config.msol_to_sol(self.liq_pool_msol_leg.amount)?;
        let total_liq_pool_value = sol_leg_available_balance + msol_leg_value;

        msg!(
            "liq_pool SOL:{}, liq_pool mSOL value:{} liq_pool_value:{}",
            sol_leg_available_balance,
            msol_leg_value,
            total_liq_pool_value
        );

        let lp_supply = self.stake_pool_config.liq_pool.lp_supply;
        let shares_for_user = shares_from_value(
            lamports, 
            total_liq_pool_value, 
            lp_supply
        )?;
        msg!("LP for user {}", shares_for_user);

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.transfer_from.to_account_info(),
                    to: self.liq_pool_sol_leg_pda.to_account_info()
                }
            ), 
            lamports
        )?;

        let user_lp_balance = self.mint_to.amount;
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(), 
                MintTo {
                    mint: self.lp_mint.to_account_info(),
                    to: self.mint_to.to_account_info(),
                    authority: self.lp_mint_authority.to_account_info()
                }, 
                &[&[
                    self.stake_pool_config.key().as_ref(),
                    LiqPool::LP_MINT_AUTHORITY_SEED,
                    &[self.stake_pool_config.liq_pool.lp_mint_authority_bump_seed]
                ]]
            ), 
            shares_for_user
        )?;
        
        self.stake_pool_config.liq_pool.on_lp_mint(shares_for_user);

        emit!(AddLiquidityEvent {
            state: self.stake_pool_config.key(),
            sol_owner: self.transfer_from.key(),
            user_sol_balance,
            user_lp_balance,
            sol_leg_balance,
            lp_supply,
            sol_added_amount: lamports,
            lp_minted: shares_for_user,
            // msol price components
            total_virtual_staked_lamports,
            msol_supply,
        });

        Ok(())
    }
}
