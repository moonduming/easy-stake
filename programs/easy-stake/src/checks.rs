use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};


#[macro_export]
macro_rules! require_lte {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 > $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)))
        }
    };
}

#[macro_export]
macro_rules! require_lt {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 >= $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
}



pub fn check_mint_authority(mint: &Mint, mint_authority: &Pubkey, field_name: &str) -> Result<()> {
    if mint.mint_authority.contains(mint_authority) {
        Ok(())
    } else {
        msg!(
            "Invalid {} mint authority {}. Expected {}",
            field_name,
            mint.mint_authority.unwrap_or_default(),
            mint_authority
        );

        Err(Error::from(ProgramError::InvalidAccountData).with_source(source!()))
    }
}


pub fn check_mint_empty(mint: &Mint, field_name: &str) -> Result<()> {
    if mint.supply > 0 {
        msg!("Non empty mint {} supply: {}", field_name, mint.supply);
        return Err(Error::from(ProgramError::InvalidArgument).with_source(source!()))
    }

    Ok(())
}


pub fn check_freeze_authority(mint: &Mint, field_name: &str) -> Result<()> {
    if mint.freeze_authority.is_some() {
        msg!("Mint {} must have freeze authority not set", field_name);
        return Err(Error::from(ProgramError::InvalidAccountData).with_source(source!()))
    }

    Ok(())
}


pub fn check_token_mint(token: &TokenAccount, mint: &Pubkey, field_name: &str) -> Result<()> {
    if token.mint != *mint {
        msg!(
            "Invalid token {} mint {}. Expected {}",
            field_name,
            token.mint,
            mint
        );
        return Err(Error::from(ProgramError::InvalidAccountData).with_source(source!()))
    }

    Ok(())
}


pub fn check_token_owner(token: &TokenAccount, owner: &Pubkey, field_name: &str) -> Result<()> {
    if token.owner != *owner {
        msg!(
            "Invalid token account {} owner {}. Expected {}",
            field_name,
            token.owner,
            owner
        );
        return Err(Error::from(ProgramError::InvalidAccountData).with_source(source!()))
    }

    Ok(())
}
