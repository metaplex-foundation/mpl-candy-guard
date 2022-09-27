use super::*;

use crate::utils::*;

/// Configurations options for the token gate. This guard only
/// allows addresses that hold the specified spl-token.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenBurn {
    pub amount: u64,
    pub mint: Pubkey,
}

impl Guard for TokenBurn {
    fn size() -> usize {
        8    // amount
        + 32 // mint
    }

    fn mask() -> u64 {
        0b1u64 << 15
    }
}

impl Condition for TokenBurn {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // retrieves the (potential) token gate account
        let token_gate_index = evaluation_context.account_cursor;
        let token_gate_account = Self::get_account_info(ctx, token_gate_index)?;
        // consumes the gate token account
        evaluation_context.account_cursor += 1;

        let account = assert_is_ata(token_gate_account, &ctx.accounts.payer.key(), &self.mint)?;

        if account.amount >= self.amount {
            let token_gate_mint = Self::get_account_info(ctx, token_gate_index + 1)?;
            // validates that we have the token_burn_authority account
            let _token_burn_authority = Self::get_account_info(ctx, token_gate_index + 2)?;
            // consumes the remaning account
            evaluation_context.account_cursor += 2;

            // is the mint account the one expected?
            assert_keys_equal(&token_gate_mint.key(), &self.mint)?;
        } else {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        evaluation_context
            .indices
            .insert("token_burn_index", token_gate_index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let token_gate_index = evaluation_context.indices["token_burn_index"];
        // the accounts have already being validated
        let token_gate_account = Self::get_account_info(ctx, token_gate_index)?;
        let token_gate_mint = Self::get_account_info(ctx, token_gate_index + 1)?;
        let token_burn_authority = Self::get_account_info(ctx, token_gate_index + 2)?;

        spl_token_burn(TokenBurnParams {
            mint: token_gate_mint.to_account_info(),
            source: token_gate_account.to_account_info(),
            amount: self.amount,
            authority: token_burn_authority.to_account_info(),
            authority_signer_seeds: None,
            token_program: ctx.accounts.token_program.to_account_info(),
        })?;

        Ok(())
    }
}