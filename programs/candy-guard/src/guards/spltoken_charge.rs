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
    fn evaluate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let amount = if evaluation_context.amount > 0 {
            evaluation_context.amount
        } else {
            self.amount
        };

        let token_account_info =
            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
        evaluation_context.remaining_account_counter += 1;
        let transfer_authority_info =
            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
        evaluation_context.remaining_account_counter += 1;

        let token_account = assert_is_ata(
            token_account_info,
            &ctx.accounts.payer.key(),
            &self.token_mint,
        )?;

        if token_account.amount < amount {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        spl_token_transfer(TokenTransferParams {
            source: token_account_info.clone(),
            destination: ctx.accounts.wallet.to_account_info(),
            authority: transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount,
        })?;

        Ok(())
    }
}
