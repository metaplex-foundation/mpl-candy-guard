use super::*;

use crate::errors::CandyGuardError;
use crate::utils::{assert_is_ata, spl_token_transfer, TokenTransferParams};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SPLTokenCharge {
    pub amount: u64,
    pub token_mint: Pubkey,
}

impl Guard for SPLTokenCharge {
    fn size() -> usize {
        std::mem::size_of::<u64>() // amount
        + 32 // token mint
    }

    fn mask() -> u64 {
        0x10u64
    }
}

impl Condition for SPLTokenCharge {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // token
        let token_account_index = evaluation_context.remaining_account_counter;
        let token_account_info = &ctx.remaining_accounts[token_account_index];
        evaluation_context.remaining_account_counter += 2;

        let token_account = assert_is_ata(
            token_account_info,
            &ctx.accounts.payer.key(),
            &self.token_mint,
        )?;

        if token_account.amount < self.amount {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        evaluation_context.amount = self.amount;
        evaluation_context.spltoken_index = token_account_index;

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.spltoken_index;
        // the accounts have already been validated
        let token_account_info = &ctx.remaining_accounts[index];
        let transfer_authority_info = &ctx.remaining_accounts[index + 1];

        spl_token_transfer(TokenTransferParams {
            source: token_account_info.clone(),
            destination: ctx.accounts.wallet.to_account_info(),
            authority: transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: evaluation_context.amount,
        })?;

        Ok(())
    }

    fn post_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // no post actions needed
        Ok(())
    }
}
