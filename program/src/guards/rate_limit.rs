use solana_program::{program::invoke_signed, system_instruction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::*;
use crate::utils::assert_keys_equal;

/// Guard to limit the rate of minting from the Candy Machine

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct RateLimit {
    pub candy_machine: Pubkey,
    pub hash: u64
}

#[account]
#[derive(Default)]
pub struct RateLimiter {
    pub candy_machine: Pubkey,
    pub hash: u64
}

impl RateLimit {
    pub const PREFIX: &'static str = "RATE_LIMIT";

    pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

}

#[derive(Hash)]
pub struct HashOfHash {
    pub(crate) hash: u64,
    pub(crate) clock: u8,
}

impl Guard for RateLimit {
    fn size() -> usize {
        std::mem::size_of::<Pubkey>() + // candy_machine
            std::mem::size_of::<u64>() // hash
    }

    fn mask() -> u64 {
        0b1u64 << 10
    }
}

impl Condition for RateLimit {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let rate_limiter = Self::get_account_info(ctx, evaluation_context.account_cursor)?;
        evaluation_context
            .indices
            .insert("rate_limiter_index", evaluation_context.account_cursor);
        evaluation_context.account_cursor += 1;

        let candy_machine_key = &ctx.accounts.candy_machine.key();
        let candy_guard_key = &ctx.accounts.candy_guard.key();

        let seeds = [
            candy_machine_key.as_ref(),
            RateLimit::PREFIX.as_bytes(),
        ];

        let signer_seeds = &seeds[..];
        let signer = &[&signer_seeds[..]];

        let (pda, _bump_seed) = Pubkey::find_program_address(signer_seeds, ctx.program_id);

        assert_keys_equal(rate_limiter.key, &pda)?;

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let rate_limiter = Self::get_account_info(ctx, evaluation_context.account_cursor)?;
        let candy_machine_key = &ctx.accounts.candy_machine.key();
        let seeds = [
            candy_machine_key.as_ref(),
            RateLimit::PREFIX.as_bytes(),
        ];
        let signer_seeds = &seeds[..];
        let signer = &[&signer_seeds[..]];
        let (pda, _bump_seed) = Pubkey::find_program_address(signer_seeds, ctx.program_id);
        assert_keys_equal(rate_limiter.key, &pda)?;

        if rate_limiter.data_is_empty() {
            let rent = Rent::get()?;
            invoke_signed(
                &system_instruction::create_account(
                    ctx.accounts.payer.key,
                    &pda,
                    rent.minimum_balance(std::mem::size_of::<u16>()),
                    self.size(),
                    &crate::ID,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    counter.to_account_info(),
                ],
                &[&signer],
            )?;

            let mut account_data = rate_limiter.try_borrow_mut_data()?;
            let mut rate_limiter = RateLimiter::try_from_slice(&account_data)?;
            // saves the changes back to the pda
            rate_limiter.hash = 0;
            let data = &mut rate_limiter.try_to_vec().unwrap();
            account_data[0..data.len()].copy_from_slice(data);

        }
        Ok(())
    }

    fn post_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let clock = solana_program::clock::Clock::get()?;
        let rate_limiter = Self::get_account_info(ctx, evaluation_context.account_cursor)?;


        let mut account_data = rate_limiter.try_borrow_mut_data()?;
        let mut rate_limiter = RateLimiter::try_from_slice(&account_data)?;
        rate_limiter.hash = rate_limiter.calculate_hash(&HashOfHash {
            hash: rate_limiter.hash,
            clock: clock.unix_timestamp as u8,
        });
        // saves the changes back to the pda
        let data = &mut rate_limiter.try_to_vec().unwrap();
        account_data[0..data.len()].copy_from_slice(data);

        Ok(())

    }
}

