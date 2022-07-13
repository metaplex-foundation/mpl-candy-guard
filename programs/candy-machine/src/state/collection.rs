use anchor_lang::prelude::*;

/// Collection PDA account.
#[account]
#[derive(Default, Debug)]
pub struct Collection {
    pub mint: Pubkey,
    pub candy_machine: Pubkey,
}
