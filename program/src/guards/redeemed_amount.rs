use super::*;

/// Configurations options for redeemed amount settings. This is a
/// guard that stop the mint once the specified amount of items
/// redeenmed is reached.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RedemeedAmount {
    pub maximum: u64,
}

impl Guard for RedemeedAmount {
    fn size() -> usize {
        8 // maximum
    }

    fn mask() -> u64 {
        0b1u64 << 11
    }
}

impl Condition for RedemeedAmount {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let candy_machine = &ctx.accounts.candy_machine;

        if candy_machine.items_redeemed >= self.maximum {
            return err!(CandyGuardError::MaximumRedeemedAmount);
        }

        Ok(())
    }
}
