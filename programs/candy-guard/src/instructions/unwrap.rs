use anchor_lang::prelude::*;
use candy_machine::CandyMachine;

use crate::state::CandyGuard;

pub fn unwrap(ctx: Context<Unwrap>) -> Result<()> {
    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();
    let candy_guard = &ctx.accounts.candy_guard;
    // PDA is the signer of the CPI
    let seeds = [
        b"candy_guard".as_ref(),
        &candy_guard.base.to_bytes(),
        &[candy_guard.bump],
    ];
    let signer = [&seeds[..]];

    let update_ix = candy_machine::cpi::accounts::SetAuthority {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        authority: candy_guard.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(candy_machine_program, update_ix, &signer);
    // candy machine update_authority CPI
    candy_machine::cpi::set_authority(cpi_ctx, ctx.accounts.authority.key())?;

    Ok(())
}

#[derive(Accounts)]
pub struct Unwrap<'info> {
    #[account(has_one = authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    #[account(mut, constraint = candy_guard.key() == candy_machine.authority)]
    pub candy_machine: Account<'info, CandyMachine>,
    /// CHECK: account constraints checked in account trait
    #[account(address = candy_machine::id())]
    pub candy_machine_program: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
