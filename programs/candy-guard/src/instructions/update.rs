use anchor_lang::prelude::*;

use crate::guards::*;
use crate::state::{CandyGuard, CandyGuardData, DATA_OFFSET};

pub fn update(ctx: Context<Update>, data: CandyGuardData) -> Result<()> {
    let info = ctx.accounts.candy_guard.to_account_info();
    // current features value
    let mut features = ctx.accounts.candy_guard.features;
    // limit to stop the update of guards
    let length = info.data_len();
    // account data
    let mut account_data = info.data.borrow_mut();

    // for each of the guards, we disable the guard (the guard will
    // be enabled if it is present in the data parameter)

    let mut offset = DATA_OFFSET + BotTax::size();
    features = BotTax::disable(features);

    if let Some(bot_tax) = data.bot_tax {
        if offset <= length {
            bot_tax.save(&mut account_data, offset - BotTax::size())?;
            features = BotTax::enable(features);
        }
    }

    offset += LiveDate::size();
    features = LiveDate::disable(features);

    if let Some(live_date) = data.live_date {
        if offset <= length {
            live_date.save(&mut account_data, offset - LiveDate::size())?;
            features = LiveDate::enable(features);
        }
    }

    ctx.accounts.candy_guard.features = features;

    Ok(())
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut, has_one = authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    pub authority: Signer<'info>,
}
