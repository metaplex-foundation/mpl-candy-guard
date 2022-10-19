use solana_program::{
    serialize_utils::{read_pubkey, read_u16},
    system_program,
};

use super::*;
use crate::{errors::CandyGuardError, utils::cmp_pubkeys};

// Number of default programs in the allow list.
static DEFAULT_PROGRAMS: &[&Pubkey] = &[
    &crate::ID,
    &mpl_candy_machine_core::ID,
    &system_program::ID,
    &spl_token::ID,
    &spl_associated_token_account::ID,
];

/// Guard that restricts the programs that can be in a mint transaction.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProgramGate {
    pub allow_list: Vec<Pubkey>,
}

impl Guard for ProgramGate {
    fn size() -> usize {
        4 + (5 * 32) // allow_list (5 addresses)
    }

    fn mask() -> u64 {
        0b1u64 << 16
    }
}

impl Condition for ProgramGate {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let ix_sysvar_account = &ctx.accounts.instruction_sysvar_account;
        let ix_sysvar_account_info = ix_sysvar_account.to_account_info();
        let ix_sysvar = ix_sysvar_account_info.data.borrow();

        let mut index = 0;
        // determines the total number of instructions in the transaction
        let num_instructions =
            read_u16(&mut index, &ix_sysvar).map_err(|_| ProgramError::InvalidAccountData)?;

        for index in 0..num_instructions {
            let mut offset = 2 + (index * 2) as usize;

            // offset for the number of accounts
            offset = read_u16(&mut offset, &ix_sysvar).unwrap() as usize;
            let num_accounts = read_u16(&mut offset, &ix_sysvar).unwrap();

            // offset for the program id
            offset += (num_accounts as usize) * (1 + 32);
            let program_id = read_pubkey(&mut offset, &ix_sysvar).unwrap();

            let mut found = false;

            for program in &self.allow_list {
                if cmp_pubkeys(&program_id, program) {
                    found = true;
                    break;
                }
            }

            if !found {
                for program in DEFAULT_PROGRAMS {
                    if !cmp_pubkeys(&program_id, program) {
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                msg!("Transaction had ix with program id {}", program_id);
                // if we reach this point, the program id was not found in the
                // allow list (the validation will fail)
                return err!(CandyGuardError::MintNotLastTransaction);
            }
        }

        Ok(())
    }
}
