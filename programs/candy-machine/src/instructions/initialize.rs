use crate::{
    constants::HIDDEN_SECTION,
    errors::CandyError,
    replace_patterns,
    state::{CandyMachine, CandyMachineData},
    utils::fixed_length_string,
};
use anchor_lang::{prelude::*, Discriminator};
use mpl_token_metadata::state::{
    MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};

pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
    let candy_machine_account = &mut ctx.accounts.candy_machine;

    let mut candy_machine = CandyMachine {
        data,
        authority: ctx.accounts.authority.key(),
        wallet: ctx.accounts.wallet.key(),
        collection: None,
        items_redeemed: 0,
        features: 0,
    };

    candy_machine.data.symbol = fixed_length_string(candy_machine.data.symbol, MAX_SYMBOL_LENGTH)?;

    if let Some(config_line) = &candy_machine.data.config_line_settings {
        // name settings
        let expected = replace_patterns(
            config_line.prefix_name.clone(),
            (candy_machine.data.items_available - 1) as usize,
        );
        if MAX_NAME_LENGTH < (expected.len() + config_line.name_length as usize) {
            return err!(CandyError::ExceededLengthError);
        }
        // uri validation
        let expected = replace_patterns(
            config_line.prefix_uri.clone(),
            (candy_machine.data.items_available - 1) as usize,
        );
        if MAX_URI_LENGTH < (expected.len() + config_line.uri_length as usize) {
            return err!(CandyError::ExceededLengthError);
        }
    } else if let Some(hidden) = &candy_machine.data.hidden_settings {
        // name settings
        let expected = replace_patterns(
            hidden.name.clone(),
            (candy_machine.data.items_available - 1) as usize,
        );
        if MAX_NAME_LENGTH < expected.len() {
            return err!(CandyError::ExceededLengthError);
        }
        // uri validation
        let expected = replace_patterns(
            hidden.uri.clone(),
            (candy_machine.data.items_available - 1) as usize,
        );
        if MAX_URI_LENGTH < expected.len() {
            return err!(CandyError::ExceededLengthError);
        }
    } else {
        return err!(CandyError::MissingConfigLinesSettings);
    }

    // (MAX_CREATOR_LIMIT - 1) because the candy machine is going to be a creator
    if candy_machine.data.creators.len() > (MAX_CREATOR_LIMIT - 1) {
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
    // payer of the transaction
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
