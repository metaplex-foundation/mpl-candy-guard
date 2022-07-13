use anchor_lang::prelude::*;

//use crate::guards::*;
use crate::state::{CandyGuard, CandyGuardData};

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.candy_guard.authority = ctx.accounts.authority.key();
    // all feature are disabled
    ctx.accounts.candy_guard.features = 0;
    /*    
    // allocates space for the features available
    let go_live = LiveDate {
        date: 7,
    };
    let encoded = bincode::serialize(&go_live).unwrap();

    let info = &mut ctx.accounts.candy_guard.to_account_info();
    let mut data = info.data.borrow_mut();

    for i in 48..48 + LiveDate::size() {
        data[i] = encoded[i - 48];
    }
    */
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = CandyGuardData::data_length())]
    pub candy_guard: Account<'info, CandyGuard>,
    /// CHECK: authority can be any account and is not written to or read
    authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
