use super::*;

use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Whitelist {
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
    pub mode: WhitelistTokenMode,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Clone, Debug)]
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
        0x4u64
    }
}

impl Condition for Whitelist {
    fn evaluate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let whitelist_token_account =
            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
        // consumes the remaning account
        evaluation_context.remaining_account_counter += 1;

        // if the user has not actually made this account, this explodes and we just check normal dates.
        // If they have, we check amount, if it's > 0 we let them use the logic
        // if 0, check normal dates.
        match assert_is_ata(
            whitelist_token_account,
            &ctx.accounts.payer.key(),
            &self.mint,
        ) {
            Ok(wta) => {
                if wta.amount > 0 {
                    if let Some(price) = self.discount_price {
                        evaluation_context.discount_price = price;
                    }

                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        let whitelist_token_mint =
                            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
                        // consumes the remaning account
                        evaluation_context.remaining_account_counter += 1;

                        let whitelist_burn_authority =
                            &ctx.remaining_accounts[evaluation_context.remaining_account_counter];
                        // consumes the remaning account
                        evaluation_context.remaining_account_counter += 1;

                        assert_keys_equal(&whitelist_token_mint.key(), &self.mint)?;

                        spl_token_burn(TokenBurnParams {
                            mint: whitelist_token_account.clone(),
                            source: whitelist_token_account.clone(),
                            amount: 1,
                            authority: whitelist_burn_authority.clone(),
                            authority_signer_seeds: None,
                            token_program: ctx.accounts.token_program.to_account_info(),
                        })?;
                    }
                } else {
                    if wta.amount == 0 && self.discount_price.is_none() && !self.presale {
                        // a non-presale whitelist with no discount price is a forced whitelist
                        // (only whitelist users can mint)
                        return err!(CandyGuardError::NoWhitelistToken);
                    }
                    // consumes the remaning accounts if needed
                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        evaluation_context.remaining_account_counter += 2;
                    }
                }
            }
            Err(_) => {
                if self.discount_price.is_none() && !self.presale {
                    // a non-presale whitelist with no discount price is a forced whitelist
                    // (only whitelist users can mint)
                    return err!(CandyGuardError::NoWhitelistToken);
                }
                // consumes the remaning accounts if needed
                if self.mode == WhitelistTokenMode::BurnEveryTime {
                    evaluation_context.remaining_account_counter += 2;
                }
            }
        }

        Ok(())
    }
}
