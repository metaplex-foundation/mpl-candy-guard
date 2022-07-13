use super::*;

#[derive(AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Whitelist {
    pub presale: bool,
}

impl Guard for Whitelist {
    fn size() -> usize {
        std::mem::size_of::<bool>() // presale
    }

    fn mask() -> u64 {
        0x4u64
    }
}

impl Condition for Whitelist {
    fn evaluate(
        &self,
        _ctx: &Context<Mint>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        Ok(())
    }
}
