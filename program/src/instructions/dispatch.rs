use anchor_lang::prelude::*;

use crate::state::{CandyGuard, SEED};

pub fn dispatch<'info>(_ctx: Context<Dispatch<'info>>) -> Result<()> {
    // let pa = CpiAccount::try_from(account)?;
    Ok(())
}

/// Withdraw the rent SOL from the candy guard account.
#[derive(Accounts)]
pub struct Dispatch<'info> {
    #[account(
        seeds = [SEED, candy_guard.base.key().as_ref()],
        bump = candy_guard.bump
    )]
    candy_guard: Account<'info, CandyGuard>,
}
