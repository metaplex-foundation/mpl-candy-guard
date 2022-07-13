pub use anchor_lang::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use solana_program::{program_memory::sol_memcmp, pubkey::PUBKEY_BYTES};

pub use crate::errors::CandyGuardError;
pub use crate::instructions::mint::*;
pub use crate::state::CandyGuardData;

pub use bot_tax::BotTax;
pub use live_date::LiveDate;
pub use whitelist::Whitelist;

mod bot_tax;
mod live_date;
mod whitelist;

pub trait Condition {
    /// Validate the condition of the guard. When the guard condition
    /// is not satisfied, it will return an error.
    fn evaluate(
        &self,
        ctx: &Context<Mint>,
        candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()>;
}

pub trait Guard: Condition + Serialize {
    /// Return the number of bytes used by the guard configuration.
    fn size() -> usize;

    /// Return the feature mask for the guard.
    fn mask() -> u64;

    /// Returns whether the guards is enabled or not on the specified features.
    fn is_enabled(features: u64) -> bool {
        features & Self::mask() > 0
    }

    /// Enable the guard on the specified `features` value.
    fn enable(features: u64) -> u64 {
        features | Self::mask()
    }

    /// Disable the guard on the specified `features` value.
    fn disable(features: u64) -> u64 {
        features & !Self::mask()
    }

    /// Serialize the guard into the specified data array.
    fn save(&self, data: &mut [u8], offset: usize) {
        let encoded = bincode::serialize(self).unwrap();

        for i in 0..encoded.len() {
            data[offset + i] = encoded[i];
        }
    }
}

pub struct EvaluationContext {
    /// The price of the asset after evaluating all guards
    pub discount_price: u64,
    /// Indicate whether the transaction was sent by the candy guard authority or not.
    pub is_authority: bool,
    /// Indicates whether the transaction started before the live date (only relevant
    /// when the [`LiveDate`](live_date::LiveDate) guard is active)
    pub is_presale: bool,
    /// The counter for the remaining account list. When a guard "consumes" one of the
    /// remaining accounts, it should increment the counter.
    pub remaining_account_counter: usize,
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}
