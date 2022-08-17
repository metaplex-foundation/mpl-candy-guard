use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::set_and_verify_collection;
use solana_program::{
    program::invoke_signed, pubkey::Pubkey, sysvar, sysvar::instructions::get_instruction_relative,
};

use crate::{cmp_pubkeys, CandyMachine};

pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
    let ixs = &ctx.accounts.instructions;
    let previous_instruction = get_instruction_relative(-1, ixs)?;
    if !cmp_pubkeys(&previous_instruction.program_id, &crate::id()) {
        msg!(
            "Transaction had ix with program id {}",
            &previous_instruction.program_id
        );
        return Ok(());
    }
    // Check if the metadata account has data if not bot fee
    if !cmp_pubkeys(ctx.accounts.metadata.owner, &mpl_token_metadata::id())
        || ctx.accounts.metadata.data_len() == 0
    {
        return Ok(());
    }

    let discriminator = &previous_instruction.data[0..8];
    if discriminator != [211, 57, 6, 167, 15, 219, 35, 251] {
        msg!("Transaction had ix with data {:?}", discriminator);
        return Ok(());
    }

    let mint_ix_accounts = previous_instruction.accounts;
    let mint_ix_cm = mint_ix_accounts[0].pubkey;
    let mint_ix_metadata = mint_ix_accounts[4].pubkey;
    let signer = mint_ix_accounts[6].pubkey;
    let candy_machine = &ctx.accounts.candy_machine;
    let metadata = ctx.accounts.metadata.key();

    if !cmp_pubkeys(&signer, &candy_machine.authority) {
        msg!(
            "Signer with pubkey {} does not match the mint ix Signer with pubkey {}",
            mint_ix_cm,
            candy_machine.authority
        );
        return Ok(());
    }
    if !cmp_pubkeys(&mint_ix_cm, &candy_machine.key()) {
        msg!(
            "Candy Machine with pubkey {} does not match the mint ix Candy Machine with pubkey {}",
            mint_ix_cm,
            candy_machine.key()
        );
        return Ok(());
    }
    if !cmp_pubkeys(&mint_ix_metadata, &metadata) {
        msg!(
            "Metadata with pubkey {} does not match the mint ix metadata with pubkey {}",
            mint_ix_metadata,
            metadata
        );
        return Ok(());
    }

    let collection_mint = ctx.accounts.collection_mint.to_account_info();

    if let Some(collection) = &candy_machine.collection {
        if !cmp_pubkeys(collection, &collection_mint.key()) {
            return Ok(());
        }
    }

    let seeds = [b"candy_machine".as_ref(), &candy_machine.key().to_bytes()];
    let (_, bump) = Pubkey::find_program_address(&seeds, &crate::ID);
    let signer = &[seeds[0], seeds[1], &[bump]];

    let set_collection_infos = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.collection.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.authority.to_account_info(),
        collection_mint.to_account_info(),
        ctx.accounts.collection_metadata.to_account_info(),
        ctx.accounts.collection_master_edition.to_account_info(),
        ctx.accounts.collection_authority_record.to_account_info(),
    ];
    invoke_signed(
        &set_and_verify_collection(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.collection.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.authority.key(),
            collection_mint.key(),
            ctx.accounts.collection_metadata.key(),
            ctx.accounts.collection_master_edition.key(),
            Some(ctx.accounts.collection_authority_record.key()),
        ),
        set_collection_infos.as_slice(),
        &[signer],
    )?;
    Ok(())
}

/// Sets and verifies the collection during a candy machine mint
#[derive(Accounts)]
pub struct SetCollection<'info> {
    #[account(has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    /// CHECK: authority can be any account and is checked in CPI
    authority: UncheckedAccount<'info>,
    // payer of the transaction
    payer: Signer<'info>,
    /// CHECK: account checked in CPI/instruction sysvar
    metadata: UncheckedAccount<'info>,
    /// CHECK: only used as a signer
    #[account(
        seeds = [b"collection".as_ref(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    collection: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    instructions: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
}
