use anchor_lang::prelude::*;
use mpl_token_metadata::{
    assertions::collection::assert_master_edition, instruction::approve_collection_authority,
    state::Metadata, state::TokenMetadataAccount,
};
use solana_program::program::invoke;

use crate::{cmp_pubkeys, CandyError, CandyMachine};

pub fn add_collection(ctx: Context<AddCollection>) -> Result<()> {
    let mint = ctx.accounts.mint.to_account_info();
    let metadata: Metadata = Metadata::from_account_info(&ctx.accounts.metadata.to_account_info())?;
    if !cmp_pubkeys(&metadata.update_authority, &ctx.accounts.authority.key()) {
        return err!(CandyError::IncorrectCollectionAuthority);
    };
    if !cmp_pubkeys(&metadata.mint, &mint.key()) {
        return err!(CandyError::MintMismatch);
    }
    let edition = ctx.accounts.edition.to_account_info();
    let authority_record = ctx.accounts.collection_authority_record.to_account_info();
    let candy_machine = &mut ctx.accounts.candy_machine;
    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    }
    if !candy_machine.data.retain_authority {
        return err!(CandyError::CandyCollectionRequiresRetainAuthority);
    }
    assert_master_edition(&metadata, &edition)?;
    if authority_record.data_is_empty() {
        let approve_collection_infos = vec![
            authority_record.clone(),
            ctx.accounts.collection.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            mint.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];
        msg!(
            "Approving collection authority for {} with new authority {}.",
            ctx.accounts.metadata.key(),
            candy_machine.key()
        );
        invoke(
            &approve_collection_authority(
                ctx.accounts.token_metadata_program.key(),
                authority_record.key(),
                ctx.accounts.collection.key(),
                ctx.accounts.authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.metadata.key(),
                *mint.key,
            ),
            approve_collection_infos.as_slice(),
        )?;
        msg!(
            "Successfully approved collection authority for collection mint {}.",
            mint.key()
        );
    }

    candy_machine.collection = Some(mint.key());

    Ok(())
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct AddCollection<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    // candy machine authority
    authority: Signer<'info>,
    // payer of the transaction
    payer: Signer<'info>,
    #[account(
        seeds = [b"collection".as_ref(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    collection: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
