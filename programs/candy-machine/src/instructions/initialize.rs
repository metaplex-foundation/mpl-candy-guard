use anchor_lang::{prelude::*, Discriminator};
use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use spl_token::state::Mint;

use crate::{
    assert_initialized, assert_owned_by, cmp_pubkeys,
    constants::HIDDEN_SECTION,
    errors::CandyError,
    state::{CandyMachine, CandyMachineData},
    utils::fixed_length_string,
};

pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
    let candy_machine_account = &mut ctx.accounts.candy_machine;

    let mut candy_machine = CandyMachine {
        data,
        authority: ctx.accounts.authority.key(),
        wallet: ctx.accounts.wallet.key(),
        token_mint: None,
        items_redeemed: 0,
    };

    if !ctx.remaining_accounts.is_empty() {
        let token_mint_info = &ctx.remaining_accounts[0];
        let _token_mint: Mint = assert_initialized(token_mint_info)?;
        let token_account: spl_token::state::Account = assert_initialized(&ctx.accounts.wallet)?;

        assert_owned_by(token_mint_info, &spl_token::id())?;
        assert_owned_by(&ctx.accounts.wallet, &spl_token::id())?;

        if !cmp_pubkeys(&token_account.mint, &token_mint_info.key()) {
            return err!(CandyError::MintMismatch);
        }

        candy_machine.token_mint = Some(*token_mint_info.key);
    }

    candy_machine.data.symbol = fixed_length_string(candy_machine.data.symbol, MAX_SYMBOL_LENGTH)?;
    // validates config line settings
    if MAX_NAME_LENGTH
        < (candy_machine.data.config_line_settings.prefix_name.len()
            + candy_machine.data.config_line_settings.name_length as usize)
    {
        return err!(CandyError::ExceededLengthError);
    }
    if MAX_URI_LENGTH
        < (candy_machine.data.config_line_settings.prefix_uri.len()
            + candy_machine.data.config_line_settings.uri_length as usize)
    {
        return err!(CandyError::ExceededLengthError);
    }

    // creators - 1 because the candy machine is going to be a creator
    if candy_machine.data.creators.len() > MAX_CREATOR_LIMIT - 1 {
        return err!(CandyError::TooManyCreators);
    }

    let mut struct_data = CandyMachine::discriminator().try_to_vec().unwrap();
    struct_data.append(&mut candy_machine.try_to_vec().unwrap());

    let mut account_data = candy_machine_account.data.borrow_mut();
    account_data[0..struct_data.len()].copy_from_slice(&struct_data);

    if candy_machine.data.hidden_settings.is_none() {
        // set the initial number of config lines
        account_data[HIDDEN_SECTION..HIDDEN_SECTION + 4].copy_from_slice(&u32::MIN.to_le_bytes());
    }

    Ok(())
}

/// Create a new candy machine.
#[derive(Accounts)]
#[instruction(data: CandyMachineData)]
pub struct Initialize<'info> {
    /// CHECK: account constraints checked in account trait
    #[account(
        zero,
        rent_exempt = skip,
        constraint = candy_machine.to_account_info().owner == program_id && candy_machine.to_account_info().data_len() >= data.get_space_for_candy()?
    )]
    candy_machine: UncheckedAccount<'info>,
    /// CHECK: wallet can be any account and is not written to or read
    wallet: UncheckedAccount<'info>,
    /// CHECK: authority can be any account and is not written to or read
    authority: UncheckedAccount<'info>,
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
