use std::io::Cursor;

use anchor_lang::prelude::*;
use borsh::BorshSchema;

use crate::{error::StakingError, require_lt};


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
    pub _reserved2: u32
}


impl List {
    pub fn new(
        discriminator: &[u8],
        item_size: u32,
        account: Pubkey,
        data: &mut [u8]
    ) -> Result<Self> {
        let result = Self {
            account,
            item_size,
            count: 0,
            _reserved1: Pubkey::default(),
            _reserved2: 0
        };
        result.init_account(discriminator, data)?;

        Ok(result)
    }

    fn init_account(&self, discriminator: &[u8], data: &mut [u8]) -> Result<()> {
        assert_eq!(self.count, 0);
        require_gte!(data.len(), 8, ErrorCode::AccountDiscriminatorNotFound);
        if data[0..8] != [0; 8] {
            return err!(ErrorCode::AccountDiscriminatorAlreadySet);
        }

        data[0..8].copy_from_slice(discriminator);

        Ok(())
    }

    pub fn capacity(&self, account_len: usize) -> Result<u32> {
        Ok(u32::try_from(
            account_len
                .checked_sub(8)
                .ok_or(ProgramError::AccountDataTooSmall)?
            )
            .map_err(|_| error!(StakingError::CalculationFailure))?
            .checked_div(self.item_size)
            .unwrap_or(std::u32::MAX)
        )
    }

    pub fn push<I: AnchorSerialize>(
        &mut self, 
        data: &mut [u8], 
        item: I
    ) -> Result<()>  {
        let capacity = self.capacity(data.len())?;
        require_lt!(self.count, capacity, StakingError::ListOverflow);

        let start = 8 + (self.count * self.item_size) as usize;
        let mut cursor = Cursor::new(&mut data[start..(start + self.item_size as usize)]);
        item.serialize(&mut cursor)?;

        self.count += 1;

        Ok(())
    }

    pub fn get<I: AnchorDeserialize>(
        &self,
        data: &[u8],
        index: u32
    ) -> Result<I> {
        require_lt!(index, self.count, StakingError::ListIndexOutOfBounds);

        let start = 8 + (index * self.item_size) as usize;

        I::deserialize(&mut &data[start..(start + self.item_size as usize)])
            .map_err(|err| {
                Error::from(ProgramError::BorshIoError(err.to_string()))
                    .with_source(source!())
            }
        )
    }

    pub fn remove(
        &mut self,
        data: &mut [u8],
        index: u32
    ) -> Result<()> {
        require_lt!(index, self.count, StakingError::ListIndexOutOfBounds);

        self.count -= 1;
        // 删除的是最后一位数据，不需要在进行处理
        if index == self.count {
            return Ok(());
        }
        
        let start = 8 + (index * self.item_size) as usize;
        let last_item_start = 8 + (self.count * self.item_size) as usize;
        data.copy_within(
            last_item_start..last_item_start + self.item_size as usize, 
            start
        );

        Ok(())
    }
}