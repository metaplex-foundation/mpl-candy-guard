use anchor_lang::prelude::*;

use instructions::*;
use state::*;

pub mod errors;
pub mod guards;
pub mod instructions;
pub mod state;

declare_id!("FEf9mMa22GxyxVKjXDUwdiYtH8isuhAsSqBSjQg9CTzu");

#[program]
pub mod candy_guard {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn mint(ctx: Context<Mint>) -> Result<()> {
        instructions::mint(ctx)
    }

    pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
        instructions::update(ctx, data)
    }
}
