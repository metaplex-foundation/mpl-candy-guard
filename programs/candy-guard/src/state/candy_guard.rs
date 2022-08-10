use anchor_lang::prelude::*;
use anchor_lang::AnchorDeserialize;

use crate::guards::*;
use candy_guard_derive::CandyGuard;

// Bytes offset for the start of the data section:
//     8 (discriminator)
//  + 32 (base)
//  +  1 (bump)
//  + 32 (authority)
//  +  8 (u64)
pub const DATA_OFFSET: usize = 8 + 32 + 1 + 32 + 8;

#[account]
#[derive(Default)]
pub struct CandyGuard {
    // Base key used to generate the PDA
    pub base: Pubkey,
    // Bump seed
    pub bump: u8,
    // Authority of the guard
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

#[derive(CandyGuard, AnchorSerialize, AnchorDeserialize, Clone, Debug)]
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
