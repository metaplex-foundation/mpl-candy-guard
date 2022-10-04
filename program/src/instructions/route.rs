use anchor_lang::prelude::*;
use mpl_candy_machine_core::CandyMachine;

use crate::state::{CandyGuard, CandyGuardData, GuardType, DATA_OFFSET, SEED};

pub fn route<'info>(
    ctx: Context<'_, '_, '_, 'info, Route<'info>>,
    args: RouteArgs,
    label: Option<String>,
) -> Result<()> {
    let candy_guard = &ctx.accounts.candy_guard;
    let account_info = &candy_guard.to_account_info();
    let account_data = account_info.data.borrow();
    // loads the active guard set
    let guard_set = CandyGuardData::active_set(&account_data[DATA_OFFSET..], label)?;

    guard_set.route(ctx, args)
}

/// Withdraw the rent SOL from the candy guard account.
#[derive(Accounts)]
#[instruction(args: RouteArgs)]
pub struct Route<'info> {
    #[account(
        seeds = [SEED, candy_guard.base.key().as_ref()],
        bump = candy_guard.bump
    )]
    pub candy_guard: Account<'info, CandyGuard>,
    #[account(
        mut,
        constraint = candy_guard.key() == candy_machine.mint_authority
    )]
    pub candy_machine: Box<Account<'info, CandyMachine>>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RouteArgs {
    pub guard: GuardType,
    pub data: Vec<u8>,
}
