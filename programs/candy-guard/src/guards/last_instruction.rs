use solana_program::{
    serialize_utils::{read_pubkey, read_u16},
    sysvar::instructions::get_instruction_relative,
};

use super::*;
use crate::{errors::CandyGuardError, utils::cmp_pubkeys};

const A_TOKEN: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LastInstruction {}

impl Guard for LastInstruction {
    fn size() -> usize {
        0
    }

    fn mask() -> u64 {
        0x20u64
    }
}

impl Condition for LastInstruction {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;
        let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();
        let instruction_sysvar = instruction_sysvar_account_info.data.borrow();
        // the next instruction after the mint
        let next_ix = get_instruction_relative(1, &instruction_sysvar_account_info);

        match next_ix {
            Ok(ix) => {
                let discriminator = &ix.data[0..8];
                let after_collection_ix =
                    get_instruction_relative(2, &instruction_sysvar_account_info);

                if !cmp_pubkeys(&ix.program_id, &crate::id())
                    || discriminator != [103, 17, 200, 25, 118, 95, 125, 61]
                    || after_collection_ix.is_ok()
                {
                    // we fail here, it is much cheaper to fail here than to allow a malicious user
                    // to add an ix at the end and then fail
                    msg!("Failing and halting due to an extra unauthorized instruction");
                    return err!(CandyGuardError::MintNotLastTransaction);
                }
            }
            Err(_) => {
                // TODO: remove these comments after collection support is merged
                //if ctx.accounts.candy_machine.collection.is_some() {
                // set_collection instruction expected
                return err!(CandyGuardError::MissingCollectionInstruction);
                //}
            }
        }

        let mut idx = 0;
        let num_instructions = read_u16(&mut idx, &instruction_sysvar)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        for index in 0..num_instructions {
            let mut current = 2 + (index * 2) as usize;
            let start = read_u16(&mut current, &instruction_sysvar).unwrap();

            current = start as usize;
            let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
            current += (num_accounts as usize) * (1 + 32);
            let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

            if !cmp_pubkeys(&program_id, &crate::id())
                && !cmp_pubkeys(&program_id, &spl_token::id())
                && !cmp_pubkeys(
                    &program_id,
                    &anchor_lang::solana_program::system_program::ID,
                )
                && !cmp_pubkeys(&program_id, &A_TOKEN)
            {
                msg!("Transaction had ix with program id {}", program_id);
                return err!(CandyGuardError::MintNotLastTransaction);
            }
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // no actions required
        Ok(())
    }

    fn post_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // no post actions needed
        Ok(())
    }
}
