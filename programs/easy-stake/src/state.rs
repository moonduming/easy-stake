use anchor_lang::prelude::*;


#[account]
pub struct StakeAccount {
    pub user_account: Pubkey,
    pub vault_account: Pubkey,
    pub rewards_account: Pubkey,
    pub locking_time: u64
}
