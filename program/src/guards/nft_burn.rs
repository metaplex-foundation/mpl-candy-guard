use super::*;

use mpl_token_metadata::{
    instruction::burn_nft,
    state::{Metadata, TokenMetadataAccount},
};
use solana_program::program::invoke;

use crate::utils::assert_keys_equal;

/// Configurations options for the nft payment. This is a payment
/// guard that charges another NFT (token) from a specific collection.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftBurn {
    pub required_collection: Pubkey,
}

impl Guard for NftBurn {
    fn size() -> usize {
        32 // required_collection
    }

    fn mask() -> u64 {
        0b1u64 << 14
    }
}

impl Condition for NftBurn {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.account_cursor;
        // validates that we received all required accounts
        let token_account_info = Self::get_account_info(ctx, index)?;
        let token_metadata = Self::get_account_info(ctx, index + 1)?;
        evaluation_context.account_cursor += 2;

        NftGate::verify_collection(
            token_account_info,
            token_metadata,
            &self.required_collection,
            ctx.accounts.payer.key,
        )?;

        let _token_edition = Self::get_account_info(ctx, index + 2)?;
        let mint_account = Self::get_account_info(ctx, index + 3)?;
        let _mint_collection_metadata = Self::get_account_info(ctx, index + 4)?;
        evaluation_context.account_cursor += 3;

        let metadata: Metadata = Metadata::from_account_info(token_metadata)?;
        // validates the account information
        assert_keys_equal(token_metadata.owner, &mpl_token_metadata::id())?;
        assert_keys_equal(&metadata.mint, mint_account.key)?;

        evaluation_context.indices.insert("nft_burn_index", index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.indices["nft_burn_index"];
        let token_account = Self::get_account_info(ctx, index)?;

        let token_metadata = Self::get_account_info(ctx, index + 1)?;
        let token_edition = Self::get_account_info(ctx, index + 2)?;
        let mint_account = Self::get_account_info(ctx, index + 3)?;
        let mint_collection_metadata = Self::get_account_info(ctx, index + 4)?;

        let burn_nft_infos = vec![
            token_metadata.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            mint_account.to_account_info(),
            token_account.to_account_info(),
            token_edition.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            mint_collection_metadata.to_account_info(),
        ];

        invoke(
            &burn_nft(
                mpl_token_metadata::ID,
                token_metadata.key(),
                ctx.accounts.payer.key(),
                mint_account.key(),
                token_account.key(),
                token_edition.key(),
                ::spl_token::ID,
                Some(mint_collection_metadata.key()),
            ),
            burn_nft_infos.as_slice(),
        )?;

        Ok(())
    }
}
