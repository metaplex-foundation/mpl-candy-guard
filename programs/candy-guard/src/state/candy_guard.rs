use anchor_lang::prelude::*;
use anchor_lang::AnchorDeserialize;
use std::cell::RefMut;

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
    //
    // available guards:
    // 1) bot tax
    // 2) live data
    // 3) lamports charge
    // 4) spltoken charge
    // 5) whitelist
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CandyGuardData {
    /// Bot tax guard (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Live data guard (controls when minting is allowed).
    pub live_date: Option<LiveDate>,
    /// Lamports charge guard (set the price for the mint in lamports).
    pub lamports_charge: Option<LamportsCharge>,
    /// Spl-token charge guard (set the price for the mint in spl-token amount).
    pub spltoken_charge: Option<SPLTokenCharge>,
    /// Whitelist guard (whitelist mint settings).
    pub whitelist: Option<Whitelist>,
}

impl CandyGuardData {
    pub fn from_data(features: u64, data: &mut RefMut<&mut [u8]>) -> Result<Self> {
        // bot tax
        let mut current = DATA_OFFSET + BotTax::size();
        let bot_tax = BotTax::load(features, data, current)?;

        // live date
        current += LiveDate::size();
        let live_date = LiveDate::load(features, data, current)?;

        // lamports charge
        current += LamportsCharge::size();
        let lamports_charge = LamportsCharge::load(features, data, current)?;

        // spltoken charge
        current += SPLTokenCharge::size();
        let spltoken_charge = SPLTokenCharge::load(features, data, current)?;

        // whitelist
        current += Whitelist::size();
        let whitelist = Whitelist::load(features, data, current)?;

        Ok(Self {
            bot_tax,
            live_date,
            lamports_charge,
            spltoken_charge,
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

        if let Some(lamports_charge) = &self.lamports_charge {
            conditions.push(lamports_charge);
        }

        if let Some(spltoken_charge) = &self.spltoken_charge {
            conditions.push(spltoken_charge);
        }

        if let Some(whitelist) = &self.whitelist {
            conditions.push(whitelist);
        }

        conditions
    }

    pub fn data_length() -> usize {
        DATA_OFFSET
            + BotTax::size()
            + LiveDate::size()
            + LamportsCharge::size()
            + SPLTokenCharge::size()
            + Whitelist::size()
    }
}
