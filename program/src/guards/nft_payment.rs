use super::*;
use crate::utils::{assert_keys_equal, spl_token_transfer, TokenTransferParams};

/// Configurations options for the nft payment. This is a payment
/// guard that charges another NFT (token) from a specific collection.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftPayment {
    pub required_collection: Pubkey,
    pub destination_ata: Pubkey,
}

impl Guard for NftPayment {
    fn size() -> usize {
        32   // required_collection
        + 32 // destination
    }

    fn mask() -> u64 {
        0b1u64 << 10
    }
}

impl Condition for NftPayment {
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

        let _transfer_authority = Self::get_account_info(ctx, index + 2)?;
        let destination_ata = Self::get_account_info(ctx, index + 3)?;
        evaluation_context.account_cursor += 2;

        assert_keys_equal(destination_ata.key, &self.destination_ata)?;

        evaluation_context
            .indices
            .insert("nft_payment_index", index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.indices["nft_payment_index"];
        let token_account = Self::get_account_info(ctx, index)?;

        let transfer_authority = Self::get_account_info(ctx, index + 2)?;
        let destination_ata = Self::get_account_info(ctx, index + 3)?;

        spl_token_transfer(TokenTransferParams {
            source: token_account.to_account_info(),
            destination: destination_ata.to_account_info(),
            authority: transfer_authority.to_account_info(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: 1, // fixed to always require 1 NFT
        })?;

        Ok(())
    }
}
