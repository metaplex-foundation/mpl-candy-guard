use super::*;
use crate::{
    errors::CandyGuardError,
    utils::{assert_is_token_account, assert_keys_equal},
};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

/// Configurations options for the nft gate. This guard restricts
/// the transaction to holders of a specified collection.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftGate {
    pub required_collection: Pubkey,
}

impl Guard for NftGate {
    fn size() -> usize {
        32 // required_collection
    }

    fn mask() -> u64 {
        0b1u64 << 13
    }
}

impl Condition for NftGate {
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

        let metadata: Metadata = Metadata::from_account_info(token_metadata)?;
        // validates the account information
        assert_keys_equal(token_metadata.owner, &mpl_token_metadata::id())?;

        let token_account = assert_is_token_account(
            token_account_info,
            &ctx.accounts.payer.key(),
            &metadata.mint,
        )?;

        if token_account.amount < 1 {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        match metadata.collection {
            Some(c) if c.verified && c.key == self.required_collection => Ok(()),
            _ => Err(CandyGuardError::InvalidNFTCollectionPayment),
        }?;

        Ok(())
    }
}
