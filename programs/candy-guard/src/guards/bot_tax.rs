use super::*;

use solana_program::{program::invoke, system_instruction};

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
    fn validate<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // the purpuse of this guard is to indicate whether the bot tax is enbled or not
        // and to store the lamports fee
        Ok(())
    }

    fn actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &EvaluationContext,
    ) -> Result<()> {
        // the purpuse of this guard is to indicate whether the bot tax is enbled or not
        // and to store the lamports fee
        Ok(())
    }
}

impl BotTax {
    pub fn punish_bots<'info>(
        &self,
        error: Error,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
    ) -> Result<()> {
        let bot_account = ctx.accounts.payer.to_account_info();
        let payment_account = ctx.accounts.candy_machine.to_account_info();
        let system_program = ctx.accounts.system_program.to_account_info();

        msg!(
            "{}, Candy Guard Botting is taxed at {:?} lamports",
            error.to_string(),
            self.lamports
        );
        let final_fee = self.lamports.min(bot_account.lamports());
        invoke(
            &system_instruction::transfer(bot_account.key, payment_account.key, final_fee),
            &[bot_account, payment_account, system_program],
        )?;
        Ok(())
    }
}
