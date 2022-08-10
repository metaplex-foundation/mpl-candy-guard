use anchor_lang::prelude::*;

use instructions::*;
use state::*;

pub mod errors;
pub mod guards;
pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("FEf9mMa22GxyxVKjXDUwdiYtH8isuhAsSqBSjQg9CTzu");

#[program]
pub mod candy_guard {
    use super::*;

    /// Create a new `CandyGuard` account.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    /// Mint an NFT from a `CandyMachine` "behind" a `CandyGuard`.
    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
        creator_bump: u8,
    ) -> Result<()> {
        instructions::mint(ctx, creator_bump)
    }

    /// Remove a `CandyGuard` from a `CandyMachine`, setting the authority to the
    /// `CandyGuard` authority.
    pub fn unwrap(ctx: Context<Unwrap>) -> Result<()> {
        instructions::unwrap(ctx)
    }

    /// Update the `CandyGuard` configuration.
    pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
        instructions::update(ctx, data)
    }

    /// Add a `CandyGuard` to a `CandyMachine`. After the guard is added, mint
    /// is only allowed through the `CandyGuard`.
    pub fn wrap(ctx: Context<Wrap>) -> Result<()> {
        instructions::wrap(ctx)
    }
}
