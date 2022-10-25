use solana_program::{
    serialize_utils::{read_pubkey, read_u16},
    system_program,
};

use super::*;
use crate::{errors::CandyGuardError, state::GuardType, utils::cmp_pubkeys};

// Default list of authorized programs.
pub static DEFAULT_PROGRAMS: &[Pubkey] = &[
    crate::ID,
    mpl_candy_machine_core::ID,
    system_program::ID,
    spl_token::ID,
    spl_associated_token_account::ID,
];

/// Guard that restricts the programs that can be in a mint transaction. The guard allows the
/// necessary programs for the mint and any other program specified in the configuration.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProgramGate {
    pub additional: Vec<Pubkey>,
}

impl Guard for ProgramGate {
    fn size() -> usize {
        4 + (5 * 32) // programs (5 addresses)
    }

    fn mask() -> u64 {
        GuardType::as_mask(GuardType::ProgramGate)
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

        let mut programs: Vec<Pubkey> =
            Vec::with_capacity(DEFAULT_PROGRAMS.len() + self.additional.len());
        programs.extend(DEFAULT_PROGRAMS);
        programs.extend(&self.additional);

        verify_programs(ix_sysvar_account_info, &programs)
    }
}

pub fn verify_programs(sysvar: AccountInfo, programs: &[Pubkey]) -> Result<()> {
    let sysvar_data = sysvar.data.borrow();

    let mut index = 0;
    // determines the total number of instructions in the transaction
    let num_instructions =
        read_u16(&mut index, &sysvar_data).map_err(|_| ProgramError::InvalidAccountData)?;

    for index in 0..num_instructions {
        let mut offset = 2 + (index * 2) as usize;

        // offset for the number of accounts
        offset = read_u16(&mut offset, &sysvar_data).unwrap() as usize;
        let num_accounts = read_u16(&mut offset, &sysvar_data).unwrap();

        // offset for the program id
        offset += (num_accounts as usize) * (1 + 32);
        let program_id = read_pubkey(&mut offset, &sysvar_data).unwrap();

        let mut found = false;

        for program in programs {
            if cmp_pubkeys(&program_id, program) {
                found = true;
                break;
            }
        }

        if !found {
            msg!("Transaction had ix with program id {}", program_id);
            // if we reach this point, the program id was not found in the
            // programs list (the validation will fail)
            return err!(CandyGuardError::UnauthorizedProgramFound);
        }
    }

    Ok(())
}
