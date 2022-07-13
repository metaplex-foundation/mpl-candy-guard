use super::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BotTax {
    pub lamports: u64,
}

impl Guard for BotTax {
    fn size() -> usize {
        std::mem::size_of::<u64>() // lamports
    }

    fn mask() -> u64 {
        0x1u64
    }
}

impl Condition for BotTax {
    fn evaluate(
        &self,
        _ctx: &Context<Mint>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // the purpuse of this guard is to indicate whether the bot tax is anebled or not
        // and to store the lamports fee
        Ok(())
    }
}
