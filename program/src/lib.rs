use anchor_lang::prelude::*;

use instructions::*;
use state::*;

pub mod errors;
pub mod guards;
pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("CnDYGRdU51FsSyLnVgSd19MCFxA4YHT5h3nacvCKMPUJ");

#[program]
pub mod candy_guard {
    use super::*;

    /// Route the transaction to a guard instruction.
    pub fn dispatch(ctx: Context<Dispatch>) -> Result<()> {
        instructions::dispatch(ctx)
    }

    /// Create a new candy guard account.
    pub fn initialize(ctx: Context<Initialize>, data: CandyGuardData) -> Result<()> {
        instructions::initialize(ctx, data)
    }

    /// Mint an NFT from a candy machine wrapped in the candy guard.
    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
        mint_args: Vec<u8>,
        label: Option<String>,
    ) -> Result<()> {
        instructions::mint(ctx, mint_args, label)
    }

    /// Remove a candy guard from a candy machine, setting the authority to the
    /// candy guard authority.
    pub fn unwrap(ctx: Context<Unwrap>) -> Result<()> {
        instructions::unwrap(ctx)
    }

    /// Update the candy guard configuration.
    pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
        instructions::update(ctx, data)
    }

    /// Withdraw the rent SOL from the candy guard account.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }

    /// Add a candy guard to a candy machine. After the guard is added, mint
    /// is only allowed through the candy guard.
    pub fn wrap(ctx: Context<Wrap>) -> Result<()> {
        instructions::wrap(ctx)
    }
}
