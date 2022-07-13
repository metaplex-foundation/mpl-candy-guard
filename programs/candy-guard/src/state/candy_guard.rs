use anchor_lang::prelude::*;
use anchor_lang::AnchorDeserialize;
use std::cell::RefMut;

use crate::errors::CandyGuardError;
use crate::guards::*;

// Bytes offset for the start of the data section:
//     8 (discriminator)
//  + 32 (authority)
//  +  8 (u64)
pub const DATA_OFFSET: usize = 8 + 32 + 8;

#[account]
#[derive(Default)]
pub struct CandyGuard {
    // Owner of the guard
    pub authority: Pubkey,
    // Guard features flag (up to 64)
    pub features: u64,
    // after this there is a flexible amount of data to serialize
    // data (struct) of the available guards; the size of the data
    // is adjustable as new guards are implemented (the account is
    // resized using realloc)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CandyGuardData {
    /// Bot tax guard (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Live data guard (controls when minting is allowed).
    pub live_date: Option<LiveDate>,
    /// Whitelist guard (whitelist mint settings).
    pub whitelist: Option<Whitelist>,
}

impl CandyGuardData {
    pub fn from_data(features: u64, data: &mut RefMut<&mut [u8]>) -> Result<Self> {
        // limit to stop trying to deserialize guards
        let length = data.len();

        // bot tax
        let mut current = DATA_OFFSET + BotTax::size();

        let bot_tax = if current <= length && BotTax::is_enabled(features) {
            let bot_tax: BotTax = deserialize(current - BotTax::size(), current, data)?;
            Some(bot_tax)
        } else {
            None
        };

        // live date
        current += LiveDate::size();

        let live_date = if current <= length && LiveDate::is_enabled(features) {
            let live_date: LiveDate = deserialize(current - LiveDate::size(), current, data)?;
            Some(live_date)
        } else {
            None
        };

        // whitelist
        current += Whitelist::size();

        let whitelist = if current <= length && Whitelist::is_enabled(features) {
            let whitelist: Whitelist = deserialize(current - Whitelist::size(), current, data)?;
            Some(whitelist)
        } else {
            None
        };

        Ok(Self {
            bot_tax,
            live_date,
            whitelist,
        })
    }

    pub fn enabled_conditions(&self) -> Vec<&dyn Condition> {
        // list of condition trait objects
        let mut conditions: Vec<&dyn Condition> = vec![];

        if let Some(bot_tax) = &self.bot_tax {
            conditions.push(bot_tax);
        }

        if let Some(live_date) = &self.live_date {
            conditions.push(live_date);
        }

        if let Some(whitelist) = &self.whitelist {
            conditions.push(whitelist);
        }

        conditions
    }

    pub fn data_length() -> usize {
        DATA_OFFSET + BotTax::size() + LiveDate::size() + Whitelist::size()
    }
}

/// Deserializes a [`guard`](crate::guards::Guard) struct from the data.
///
/// # Arguments
///
/// * `from` - start offset on the data array
/// * `to` - end offset on the data array
/// * `data` - data array
///
/// # Errors
///
/// - [`DeserializationError`](crate::errors::CandyGuardError::DeserializationError) when
///    the guard cannot be read from the data.
fn deserialize<'a, T>(from: usize, to: usize, data: &'a [u8]) -> Result<T>
where
    T: Guard + Deserialize<'a>,
{
    if let Ok(decoded) = bincode::deserialize::<T>(&data[from..to]) {
        Ok(decoded)
    } else {
        err!(CandyGuardError::DeserializationError)
    }
}
