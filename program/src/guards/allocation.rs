use solana_program::{program::invoke_signed, system_instruction};

use super::*;
use crate::{
    state::GuardType,
    utils::{assert_keys_equal, assert_owned_by},
};

/// Gaurd to specify the maximum number of mints in a guard set.
///
/// List of accounts required:
///
///   0. `[writable]` Mint tracker PDA. The PDA is derived
///                   using the seed `["allocation", allocation id,
///                   candy guard pubkey, candy machine pubkey]`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Allocation {
    /// Unique identifier of the allocation.
    pub id: u8,
    /// The size of the allocation.
    pub size: u32,
}

/// PDA to track the number of mints.
#[account]
#[derive(Default)]
pub struct MintTracker {
    pub count: u32,
}

impl Guard for Allocation {
    fn size() -> usize {
        1   // id
        + 4 // count
    }

    fn mask() -> u64 {
        GuardType::as_mask(GuardType::Allocation)
    }

    /// Instruction to initialize the allocation PDA.
    ///
    /// List of accounts required:
    ///
    ///   0. `[writable]` Pda to track the number of mints (seeds `["allocation", allocation id,
    ///                   candy guard pubkey, candy machine pubkey]`).
    ///   1. `[]` System program account.
    fn instruction<'info>(
        ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
        route_context: RouteContext<'info>,
        _data: Vec<u8>,
    ) -> Result<()> {
        msg!("Instruction: Initialize (Allocation guard)");

        if route_context.candy_guard.is_none() || route_context.candy_machine.is_none() {
            return err!(CandyGuardError::Uninitialized);
        }

        let allocation_id = if let Some(guard_set) = &route_context.guard_set {
            if let Some(allocation) = &guard_set.allocation {
                allocation.id
            } else {
                return err!(CandyGuardError::AllocationGuardNotEnabled);
            }
        } else {
            return err!(CandyGuardError::AllocationGuardNotEnabled);
        };

        let allocation = get_account_info(ctx, 0)?;
        let _system_program = get_account_info(ctx, 1)?;

        if allocation.data_is_empty() {
            let candy_guard_key = &ctx.accounts.candy_guard.key();
            let candy_machine_key = &ctx.accounts.candy_machine.key();

            let seeds = [
                b"allocation".as_ref(),
                &[allocation_id],
                candy_guard_key.as_ref(),
                candy_machine_key.as_ref(),
            ];
            let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);

            let signer = [
                b"allocation".as_ref(),
                &[allocation_id],
                candy_guard_key.as_ref(),
                candy_machine_key.as_ref(),
                &[bump],
            ];
            let rent = Rent::get()?;

            invoke_signed(
                &system_instruction::create_account(
                    &ctx.accounts.payer.key(),
                    &pda,
                    rent.minimum_balance(std::mem::size_of::<u32>()),
                    std::mem::size_of::<u32>() as u64,
                    &crate::ID,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    allocation.to_account_info(),
                ],
                &[&signer],
            )?;
        } else {
            // if it an existing account, make sure it has the correct ownwer
            assert_owned_by(allocation, &crate::ID)?;
        }

        let mut account_data = allocation.try_borrow_mut_data()?;
        let mut mint_tracker = MintTracker::try_from_slice(&account_data)?;
        // initial count is always zero
        mint_tracker.count = 0;
        // saves the changes back to the pda
        let data = &mut mint_tracker.try_to_vec().unwrap();
        account_data[0..data.len()].copy_from_slice(data);

        Ok(())
    }
}

impl Condition for Allocation {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let allocation = get_account_info(ctx, evaluation_context.account_cursor)?;
        evaluation_context
            .indices
            .insert("allocation_index", evaluation_context.account_cursor);
        evaluation_context.account_cursor += 1;

        let candy_guard_key = &ctx.accounts.candy_guard.key();
        let candy_machine_key = &ctx.accounts.candy_machine.key();

        let seeds = [
            b"allocation".as_ref(),
            &[self.id],
            candy_guard_key.as_ref(),
            candy_machine_key.as_ref(),
        ];
        let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

        assert_keys_equal(allocation.key, &pda)?;

        if allocation.data_is_empty() {
            // sanity check: if the limit is set to less than 1 we cannot proceed
            return err!(CandyGuardError::AllocationNotInitialized);
        } else {
            // make sure the account has the correct owner
            assert_owned_by(allocation, &crate::ID)?;
        }

        let account_data = allocation.try_borrow_data()?;
        let mint_tracker = MintTracker::try_from_slice(&account_data)?;

        if mint_tracker.count >= self.size {
            return err!(CandyGuardError::AllocationLimitReached);
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
        let allocation = get_account_info(ctx, evaluation_context.indices["allocation_index"])?;
        let mut account_data = allocation.try_borrow_mut_data()?;
        let mut mint_tracker = MintTracker::try_from_slice(&account_data)?;

        mint_tracker.count += 1;
        // saves the changes back to the pda
        let data = &mut mint_tracker.try_to_vec().unwrap();
        account_data[0..data.len()].copy_from_slice(data);

        Ok(())
    }
}
