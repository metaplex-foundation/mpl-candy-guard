use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use arrayref::array_ref;
use mpl_token_metadata::instruction::{
    create_master_edition_v3, create_metadata_accounts_v2, update_metadata_accounts_v2,
};
use solana_program::{
    program::invoke_signed,
    serialize_utils::{read_pubkey, read_u16},
    sysvar,
    sysvar::instructions::get_instruction_relative,
};

use crate::{
    constants::{A_TOKEN, CUPCAKE_ID, EMPTY_STR, GUMDROP_ID, HIDDEN_SECTION, PREFIX},
    utils::*,
    CandyError, CandyMachine, ConfigLine,
};

pub fn mint<'info>(ctx: Context<'_, '_, '_, 'info, Mint<'info>>, creator_bump: u8) -> Result<()> {
    // (1) validation

    if !ctx.accounts.metadata.data_is_empty() {
        return err!(CandyError::MetadataAccountMustBeEmpty);
    }

    // the candy machine authority has "super-powers", so no need to validate instructions
    if !cmp_pubkeys(
        &ctx.accounts.authority.key(),
        &ctx.accounts.candy_machine.authority,
    ) {
        let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;
        let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();
        let instruction_sysvar = instruction_sysvar_account_info.data.borrow();
        let current_ix = get_instruction_relative(0, &instruction_sysvar_account_info).unwrap();

        // restricting who can call candy machine via CPI
        if !cmp_pubkeys(&current_ix.program_id, &crate::id())
            && !cmp_pubkeys(&current_ix.program_id, &GUMDROP_ID)
            && !cmp_pubkeys(&current_ix.program_id, &CUPCAKE_ID)
        {
            return err!(CandyError::SuspiciousTransaction);
        }
        let next_ix = get_instruction_relative(1, &instruction_sysvar_account_info);

        match next_ix {
            Ok(ix) => {
                let discriminator = &ix.data[0..8];
                let after_collection_ix =
                    get_instruction_relative(2, &instruction_sysvar_account_info);
                if !cmp_pubkeys(&ix.program_id, &crate::id())
                    || discriminator != [103, 17, 200, 25, 118, 95, 125, 61]
                    || after_collection_ix.is_ok()
                {
                    // we fail here, it is much cheaper to fail here than to allow a malicious user to
                    // add an ix at the end and then fail
                    msg!("Failing and halting due to an extra unauthorized instruction");
                    return err!(CandyError::SuspiciousTransaction);
                }
            }
            Err(_) => {
                // TODO: collection support missing
                return err!(CandyError::MissingSetCollectionDuringMint);
            }
        }

        let mut idx = 0;
        let num_instructions = read_u16(&mut idx, &instruction_sysvar)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        for index in 0..num_instructions {
            let mut current = 2 + (index * 2) as usize;
            let start = read_u16(&mut current, &instruction_sysvar).unwrap();

            current = start as usize;
            let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
            current += (num_accounts as usize) * (1 + 32);
            let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

            if !cmp_pubkeys(&program_id, &crate::id())
                && !cmp_pubkeys(&program_id, &spl_token::id())
                && !cmp_pubkeys(
                    &program_id,
                    &anchor_lang::solana_program::system_program::ID,
                )
                && !cmp_pubkeys(&program_id, &A_TOKEN)
            {
                msg!("Transaction had ix with program id {}", program_id);
                return err!(CandyError::SuspiciousTransaction);
            }
        }
    }

    let candy_machine = &mut ctx.accounts.candy_machine;
    let candy_machine_creator = &ctx.accounts.candy_machine_creator;

    // are there items to be minted?
    if candy_machine.items_redeemed >= candy_machine.data.items_available {
        return err!(CandyError::CandyMachineEmpty);
    }

    // (2) selecting an item to mint

    let recent_slothashes = &ctx.accounts.recent_slothashes;
    let data = recent_slothashes.data.borrow();
    let most_recent = array_ref![data, 12, 8];

    let numerator = u64::from_le_bytes(*most_recent);
    let remainder: usize = numerator
        .checked_rem(candy_machine.data.items_available - candy_machine.items_redeemed)
        .ok_or(CandyError::NumericalOverflowError)? as usize;

    let config_line = get_config_line(candy_machine, remainder, candy_machine.items_redeemed)?;

    candy_machine.items_redeemed = candy_machine
        .items_redeemed
        .checked_add(1)
        .ok_or(CandyError::NumericalOverflowError)?;

    // (3) minting

    let cm_key = candy_machine.key();
    let authority_seeds = [PREFIX.as_bytes(), cm_key.as_ref(), &[creator_bump]];

    let mut creators: Vec<mpl_token_metadata::state::Creator> =
        vec![mpl_token_metadata::state::Creator {
            address: candy_machine_creator.key(),
            verified: true,
            share: 0,
        }];

    for c in &candy_machine.data.creators {
        creators.push(mpl_token_metadata::state::Creator {
            address: c.address,
            verified: false,
            share: c.percentage_share,
        });
    }

    let metadata_infos = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        candy_machine_creator.to_account_info(),
    ];

    let master_edition_infos = vec![
        ctx.accounts.master_edition.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        candy_machine_creator.to_account_info(),
    ];
    invoke_signed(
        &create_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            candy_machine_creator.key(),
            config_line.name,
            candy_machine.data.symbol.clone(),
            config_line.uri,
            Some(creators),
            candy_machine.data.seller_fee_basis_points,
            true,
            candy_machine.data.is_mutable,
            None,
            None,
        ),
        metadata_infos.as_slice(),
        &[&authority_seeds],
    )?;
    invoke_signed(
        &create_master_edition_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.master_edition.key(),
            ctx.accounts.mint.key(),
            candy_machine_creator.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.payer.key(),
            Some(candy_machine.data.max_supply),
        ),
        master_edition_infos.as_slice(),
        &[&authority_seeds],
    )?;

    let mut new_update_authority = Some(candy_machine.authority);

    if !candy_machine.data.retain_authority {
        new_update_authority = Some(ctx.accounts.update_authority.key());
    }
    invoke_signed(
        &update_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            candy_machine_creator.key(),
            new_update_authority,
            None,
            Some(true),
            if !candy_machine.data.is_mutable {
                Some(false)
            } else {
                None
            },
        ),
        &[
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            candy_machine_creator.to_account_info(),
        ],
        &[&authority_seeds],
    )?;

    Ok(())
}

pub fn get_config_line(
    candy_machine: &Account<'_, CandyMachine>,
    index: usize,
    mint_number: u64,
) -> Result<ConfigLine> {
    if let Some(hs) = &candy_machine.data.hidden_settings {
        return Ok(ConfigLine {
            name: replace_patterns(hs.name.clone(), mint_number as usize),
            uri: replace_patterns(hs.uri.clone(), mint_number as usize),
        });
    }
    let settings = if let Some(settings) = &candy_machine.data.config_line_settings {
        settings
    } else {
        return err!(CandyError::MissingConfigLinesSettings);
    };

    let account_info = candy_machine.to_account_info();
    let mut account_data = account_info.data.borrow_mut();

    // (1) determine the mint index (index is a random index on the available indices array)

    let items_available = candy_machine.data.items_available as u64;
    let indices_start = HIDDEN_SECTION
        + 4
        + (items_available as usize) * candy_machine.data.get_config_line_size()
        + 4
        + ((items_available
            .checked_div(8)
            .ok_or(CandyError::NumericalOverflowError)?
            + 1) as usize)
        + 4;
    // calculates the mint index and retrieves the value at that position
    let mint_index = indices_start + index * 4;
    let value_to_use = u32::from_le_bytes(*array_ref![account_data, mint_index, 4]) as usize;
    // calculates the last available index and retrieves the value at that position
    let last_index = indices_start + ((items_available - mint_number - 1) * 4) as usize;
    let last_value = u32::from_le_bytes(*array_ref![account_data, last_index, 4]);
    // swap-remove: this guarantees that we remove the used mint index from the available array
    // in a constant time O(1) no matter how big the indices array is
    account_data[mint_index..mint_index + 4].copy_from_slice(&u32::to_le_bytes(last_value));

    // (2) retrieve the config line at the mint_index position

    let mut position =
        HIDDEN_SECTION + 4 + value_to_use * candy_machine.data.get_config_line_size();
    let name_length = settings.name_length as usize;
    let uri_length = settings.uri_length as usize;

    // name
    let name = if name_length > 0 {
        let name_slice: &mut [u8] = &mut account_data[position..position + name_length];
        unsafe { String::from_utf8_unchecked(name_slice.to_vec()) }
    } else {
        EMPTY_STR.to_string()
    };
    // uri
    position += name_length;
    let uri = if uri_length > 0 {
        let uri_slice: &mut [u8] = &mut account_data[position..position + uri_length];
        unsafe { String::from_utf8_unchecked(uri_slice.to_vec()) }
    } else {
        EMPTY_STR.to_string()
    };

    let complete_name = settings.prefix_name.clone() + &name;
    let complete_uri = settings.prefix_uri.clone() + &uri;

    Ok(ConfigLine {
        name: replace_patterns(complete_name, value_to_use),
        uri: replace_patterns(complete_uri, value_to_use),
    })
}

/// Mint a new NFT pseudo-randomly from the config array.
#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct Mint<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,
    /// CHECK: account constraints checked in account trait
    #[account(seeds=[PREFIX.as_bytes(), candy_machine.key().as_ref()], bump=creator_bump)]
    candy_machine_creator: UncheckedAccount<'info>,
    // candy machine authority
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    // With the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    /// CHECK: account checked in CPI
    #[account(mut)]
    metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    mint: UncheckedAccount<'info>,
    mint_authority: Signer<'info>,
    update_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    recent_slothashes: UncheckedAccount<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    instruction_sysvar_account: UncheckedAccount<'info>,
}
