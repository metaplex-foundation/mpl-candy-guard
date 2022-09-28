use crate::utils::cmp_pubkeys;

use super::*;

/// Guard that restricts access to a specific address.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AddressGate {
    address: Pubkey,
}

impl Guard for AddressGate {
    fn size() -> usize {
        32 // address
    }

    fn mask() -> u64 {
        0b1u64 << 12
    }
}

impl Condition for AddressGate {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if !cmp_pubkeys(&ctx.accounts.payer.key(), &self.address) {
            return err!(CandyGuardError::AddressNotAuthorized);
        }

        Ok(())
    }
}
