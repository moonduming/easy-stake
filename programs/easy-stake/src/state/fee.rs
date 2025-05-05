use std::fmt::{Display, Formatter};

use anchor_lang::prelude::*;

use crate::{error::StakingError, require_lte};


#[derive(
    Clone, Copy, Debug, Default, AnchorSerialize, AnchorDeserialize, 
    PartialEq, Eq, PartialOrd, Ord
)]
pub struct Fee {
    pub basis_points: u32,
}


impl Display for Fee {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}.{:0>2}%", 
            self.basis_points / 100, 
            self.basis_points % 100
        )
    }
}


impl Fee {
    pub const MAX_BASIS_POINTS: u32 = 10_000;

    pub const fn from_basis_points(basis_points: u32) -> Self {
        Self { basis_points }
    }

    pub fn check(&self) -> Result<()> {
        require_lte!(
            self.basis_points, 
            Self::MAX_BASIS_POINTS, 
            StakingError::BasisPointCentsOverflow
        );

        Ok(())
    }

    pub fn apply(&self, lamports: u64) -> u64 {
        (lamports as u128 * self.basis_points as u128 / Self::MAX_BASIS_POINTS as u128) as u64
    }
}
