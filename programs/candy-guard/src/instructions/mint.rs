use anchor_lang::prelude::*;

use crate::guards::{cmp_pubkeys, EvaluationContext};
use crate::state::{CandyGuard, CandyGuardData};

pub fn mint(ctx: Context<Mint>) -> Result<()> {
    let info = ctx.accounts.candy_guard.to_account_info();
    let mut data = info.data.borrow_mut();

    let candy_guard_data = CandyGuardData::from_data(ctx.accounts.candy_guard.features, &mut data)?;
    let conditions = candy_guard_data.enabled_conditions();
    // context for this transaction
    let mut evaluation_context = EvaluationContext {
        discount_price: 0,
        is_authority: cmp_pubkeys(
            &ctx.accounts.candy_guard.authority,
            &ctx.accounts.payer.key(),
        ),
        is_presale: false,
        remaining_account_counter: 0,
    };

    for condition in conditions {
        if let Err(error) = condition.evaluate(&ctx, &candy_guard_data, &mut evaluation_context) {
            return if let Some(_bot_tax) = candy_guard_data.bot_tax {
                // apply bot tax using bot_tax.lamports as fee
                msg!("bot_tax applied");
                Ok(())
            } else {
                Err(error)
            };
        }
    }

    // all guards are successful, forward the transaction to Candy Machine using the
    // price from the evaluation_context

    Ok(())
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub candy_guard: Account<'info, CandyGuard>,
    pub payer: Signer<'info>,
}
