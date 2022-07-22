use anchor_lang::prelude::*;

//use crate::guards::*;
use crate::state::{CandyGuard, CandyGuardData};

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.candy_guard.authority = ctx.accounts.authority.key();
    // all feature are disabled
    ctx.accounts.candy_guard.features = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        constraint = authority.key == payer.key,
        space = CandyGuardData::data_length()
    )]
    pub candy_guard: Account<'info, CandyGuard>,
    /// CHECK: authority == payer
    authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
