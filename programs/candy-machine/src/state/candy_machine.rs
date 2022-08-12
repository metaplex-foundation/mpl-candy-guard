use anchor_lang::prelude::*;

use super::candy_machine_data::CandyMachineData;

/// Candy machine state and config data.
#[account]
#[derive(Default, Debug)]
pub struct CandyMachine {
    pub features: u64,
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Option<Pubkey>,
    pub items_redeemed: u64,
    pub data: CandyMachineData,
    // hidden data section to avoid deserialisation:
    //
    // - (u32) how many actual lines of data there are currently (eventually
    //   equals items available)
    // - (ConfigLine * items_available) lines and lines of name + uri data
    // - (ceil(item_available / 8) + 1) bit mask to keep track of which ConfigLines
    //   have been added
    // - (u32 * items_available) mint indices
}

/// Config line struct for storing asset (NFT) data pre-mint.
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine {
    /// Name of the asset.
    pub name: String,
    /// URI to JSON metadata.
    pub uri: String,
}
