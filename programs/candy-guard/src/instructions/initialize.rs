use anchor_lang::prelude::*;

//use crate::guards::*;
use crate::state::{CandyGuard, CandyGuardData};

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let candy_guard = &mut ctx.accounts.candy_guard;
    candy_guard.base = ctx.accounts.base.key();
    candy_guard.bump = *ctx.bumps.get("candy_guard").unwrap();
    candy_guard.authority = ctx.accounts.authority.key();
    // all feature are disabled
    candy_guard.features = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = CandyGuardData::data_length(),
        seeds = [b"candy_guard", base.key().as_ref()],
        bump
    )]
    pub candy_guard: Account<'info, CandyGuard>,
    // Base key of the candy guard PDA
    #[account(mut)]
    pub base: Signer<'info>,
    /// CHECK: authority can be any account and is not written to or read
    authority: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
