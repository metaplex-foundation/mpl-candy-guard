use super::*;

use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Whitelist {
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
    pub mode: WhitelistTokenMode,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Debug)]
pub enum WhitelistTokenMode {
    BurnEveryTime,
    NeverBurn,
}

impl Guard for Whitelist {
    fn size() -> usize {
        32                               // mint
        + std::mem::size_of::<bool>()    // presale
        + 1 + std::mem::size_of::<u64>() // option + discount_price
        + 1 // mode
    }

    fn mask() -> u64 {
        0b1u64 << 5
    }
}

impl Condition for Whitelist {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // retrieves the (potential) whitelist token account
        let whitelist_index = evaluation_context.remaining_account_counter;
        let whitelist_token_account = &ctx.remaining_accounts[whitelist_index];
        // consumes the whitelist token account
        evaluation_context.remaining_account_counter += 1;

        // if the user has not actually made this account, this explodes and we just
        // check normal dates. if they have, we check amount: if it's > 0 we let them
        // use the logic; if 0, check normal dates.
        match assert_is_ata(
            whitelist_token_account,
            &ctx.accounts.payer.key(),
            &self.mint,
        ) {
            Ok(wta) => {
                // if the amount is greater than 0, the user is allowed to mint
                // we only need to check whether there is a discount price or not
                // and burn the token if needed
                if wta.amount > 0 {
                    if let Some(price) = self.discount_price {
                        // user will pay the discount price (either lamports or spl-token
                        // amount)
                        if candy_guard_data.lamports_charge.is_some() {
                            evaluation_context.lamports = price;
                        } else if candy_guard_data.spltoken_charge.is_some() {
                            evaluation_context.amount = price;
                        }
                    }
                    // should we burn the token?
                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        let whitelist_token_mint =
                            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
                        // consumes the remaning account
                        evaluation_context.remaining_account_counter += 2;
                        // is the mint account the one expected?
                        assert_keys_equal(&whitelist_token_mint.key(), &self.mint)?;

                        evaluation_context.whitelist_index = whitelist_index;
                    }
                    // user is whitelisted
                    evaluation_context.whitelist = true;
                } else {
                    // if the user does not have balance, we need to check whether the mint
                    // is in presale period or limited to only whitelist users
                    if wta.amount == 0
                        && ((self.discount_price.is_none() && !self.presale)
                            || evaluation_context.is_presale)
                        && !evaluation_context.is_authority
                    {
                        // (only whitelist users can mint) a non-presale whitelist with no discount
                        // price is a forced whitelist or we are in presale period
                        return err!(CandyGuardError::NoWhitelistToken);
                    }
                    // no presale period, consumes the remaning accounts if needed
                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        evaluation_context.remaining_account_counter += 2;
                    }
                }
            }
            Err(_) => {
                // no token, we need to check whether the mint is in presale period or limited
                // to only whitelist users
                if ((self.discount_price.is_none() && !self.presale)
                    || evaluation_context.is_presale)
                    && !evaluation_context.is_authority
                {
                    // (only whitelist users can mint) a non-presale whitelist with no discount
                    // price is a forced whitelist or if we are in presale period
                    return err!(CandyGuardError::NoWhitelistToken);
                }
                // no presale period, consumes the remaning accounts if needed
                if self.mode == WhitelistTokenMode::BurnEveryTime {
                    evaluation_context.remaining_account_counter += 2;
                }
            }
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if evaluation_context.whitelist && self.mode == WhitelistTokenMode::BurnEveryTime {
            let index = evaluation_context.whitelist_index;
            // the accounts have already being validated
            let whitelist_token_account = &ctx.remaining_accounts[index];
            let whitelist_burn_authority = &ctx.remaining_accounts[index + 2];

            spl_token_burn(TokenBurnParams {
                mint: whitelist_token_account.clone(),
                source: whitelist_token_account.clone(),
                amount: 1,
                authority: whitelist_burn_authority.clone(),
                authority_signer_seeds: None,
                token_program: ctx.accounts.token_program.to_account_info(),
            })?;
        }
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
