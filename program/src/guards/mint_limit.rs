use solana_program::{program::invoke_signed, system_instruction};

use super::*;
use crate::utils::assert_keys_equal;

/// Configurations options for mint limit.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MintLimit {
    /// Unique identifier of the mint limit.
    pub id: u8,
    /// Limit of mints per individual address.
    pub limit: u16,
}

/// PDA to track the number of mints for an individual address.
#[account]
#[derive(Default)]
pub struct MintCounter {
    pub count: u16,
}

impl Guard for MintLimit {
    fn size() -> usize {
        1   // id
        + 2 // limit
    }

    fn mask() -> u64 {
        0b1u64 << 9
    }
}

impl Condition for MintLimit {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let counter = Self::get_account_info(ctx, evaluation_context.account_cursor)?;
        evaluation_context
            .indices
            .insert("mintlimit_index", evaluation_context.account_cursor);
        evaluation_context.account_cursor += 1;

        let user = ctx.accounts.payer.key();
        let candy_guard_key = &ctx.accounts.candy_guard.key();
        let candy_machine_key = &ctx.accounts.candy_machine.key();

        let seeds = [
            &[self.id],
            user.as_ref(),
            candy_guard_key.as_ref(),
            candy_machine_key.as_ref(),
        ];
        let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

        assert_keys_equal(counter.key, &pda)?;

        if !counter.data_is_empty() {
            let account_data = counter.data.borrow();
            let mint_counter = MintCounter::try_from_slice(&account_data)?;

            if mint_counter.count >= self.limit {
                return err!(CandyGuardError::AllowedMintLimitReached);
            }
        } else if self.limit < 1 {
            // sanity check: if the limit is set to less than 1 we cannot proceed
            return err!(CandyGuardError::AllowedMintLimitReached);
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let counter =
            Self::get_account_info(ctx, evaluation_context.indices["mintlimit_index"])?;

        let user = ctx.accounts.payer.key();
        let candy_guard_key = &ctx.accounts.candy_guard.key();
        let candy_machine_key = &ctx.accounts.candy_machine.key();

        if counter.data_is_empty() {
            let seeds = [
                &[self.id],
                user.as_ref(),
                candy_guard_key.as_ref(),
                candy_machine_key.as_ref(),
            ];
            let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);

            let rent = Rent::get()?;
            let signer = [
                &[self.id],
                user.as_ref(),
                candy_guard_key.as_ref(),
                candy_machine_key.as_ref(),
                &[bump],
            ];

            invoke_signed(
                &system_instruction::create_account(
                    &ctx.accounts.payer.key,
                    &pda,
                    rent.minimum_balance(std::mem::size_of::<u16>()),
                    std::mem::size_of::<u16>() as u64,
                    &crate::ID,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    counter.to_account_info(),
                ],
                &[&signer],
            )?;
        }

        let mut account_data = counter.try_borrow_mut_data()?;
        let mut mint_counter = MintCounter::try_from_slice(&account_data)?;
        mint_counter.count += 1;
        // saves the changes back to the pda
        let data = &mut mint_counter.try_to_vec().unwrap();
        account_data[0..data.len()].copy_from_slice(data);

        Ok(())
    }
}
