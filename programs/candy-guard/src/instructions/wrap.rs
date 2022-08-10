use anchor_lang::prelude::*;
use candy_machine::CandyMachine;

use crate::state::CandyGuard;

pub fn wrap(ctx: Context<Wrap>) -> Result<()> {
    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();
    let update_ix = candy_machine::cpi::accounts::SetAuthority {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(candy_machine_program, update_ix);
    // candy machine update_authority CPI
    candy_machine::cpi::set_authority(cpi_ctx, ctx.accounts.candy_guard.key())?;

    Ok(())
}

#[derive(Accounts)]
pub struct Wrap<'info> {
    #[account(has_one = authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    #[account(mut, has_one = authority)]
    pub candy_machine: Account<'info, CandyMachine>,
    /// CHECK: account constraints checked in account trait
    #[account(address = candy_machine::id())]
    pub candy_machine_program: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
