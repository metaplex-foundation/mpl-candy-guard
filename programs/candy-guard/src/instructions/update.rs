use anchor_lang::prelude::*;

use crate::state::{CandyGuard, CandyGuardData};

pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
    let info = ctx.accounts.candy_guard.to_account_info();
    // account data
    let mut account_data = info.data.borrow_mut();
    // save the guards information to the account data and stores
    // the updated feature flag
    ctx.accounts.candy_guard.features = data.to_data(&mut account_data)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(data: CandyGuardData)]
pub struct Update<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"candy_guard", candy_guard.base.key().as_ref()],
        bump = candy_guard.bump
    )]
    pub candy_guard: Account<'info, CandyGuard>,
    pub authority: Signer<'info>,
}
