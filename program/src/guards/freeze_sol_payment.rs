use super::*;

use anchor_lang::AccountsClose;
use mpl_candy_machine_core::CandyMachine;
use mpl_token_metadata::instruction::{freeze_delegated_account, thaw_delegated_account};
use solana_program::{
    program::{invoke, invoke_signed},
    program_pack::Pack,
    system_instruction, system_program,
};
use spl_token::{
    instruction::{approve, revoke},
    state::Account as TokenAccount,
};

use crate::{
    errors::CandyGuardError,
    utils::{assert_is_ata, assert_keys_equal, cmp_pubkeys},
};

/// Guard that charges an amount in SOL (lamports) for the mint with a freeze period.
///
/// List of accounts required:
///
///   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
///           destination pubkey, candy guard pubkey, candy machine pubkey]`).
///   1. `[]` Associate token account of the NFT (seeds `[payer pubkey, nft mint pubkey]`).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FreezeSolPayment {
    pub lamports: u64,
    pub destination: Pubkey,
}

impl Guard for FreezeSolPayment {
    fn size() -> usize {
        8    // lamports
        + 32 // destination
    }

    fn mask() -> u64 {
        0b1u64 << 16
    }

    /// Instructions to interact with the freeze feature:
    ///
    ///  * initialize
    ///  * thaw
    ///  * unlock funds
    fn instruction<'info>(
        ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
        guard_set: &GuardSet,
        data: Vec<u8>,
    ) -> Result<()> {
        // determines the instruction to execute
        let instruction: FreezeInstruction =
            if let Ok(instruction) = FreezeInstruction::try_from_slice(&data[0..1]) {
                instruction
            } else {
                return err!(CandyGuardError::MissingFreezeInstruction);
            };

        let freeze_guard = if let Some(freeze_guard) = &guard_set.freeze_sol_payment {
            freeze_guard
        } else {
            return err!(CandyGuardError::FreezeGuardNotEnabled);
        };

        match instruction {
            // Initializes the freeze escrow PDA.
            //
            // List of accounts required:
            //
            //   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
            //                   destination pubkey, candy guard pubkey, candy machine pubkey]`).
            //   1. `[signer]` Candy Guard authority.
            //   2. `[]` System program account.
            FreezeInstruction::Initialize => {
                msg!("FreezeSolPayment: initialize");
                initialize_freeze(ctx, data, freeze_guard.destination)
            }
            // Thaw an eligible NFT.
            //
            // List of accounts required:
            //
            //   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
            //                   destination pubkey, candy guard pubkey, candy machine pubkey]`).
            //   1. `[]` Mint account for the NFT.
            //   2. `[]` Address of the owner of the NFT.
            //   3. `[writable]` Associate token account of the NFT.
            //   4. `[]` Master Edition account of the NFT.
            //   5. `[]` spl-token program ID.
            //   6. `[]` Metaplex Token Metadata program ID.
            FreezeInstruction::Thaw => {
                msg!("FreezeSolPayment: thaw");
                thaw_nft(ctx, data, freeze_guard.destination)
            }
            // Unlocks frozen funds.
            //
            // List of accounts required:
            //
            //   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
            //                   destination pubkey, candy guard pubkey, candy machine pubkey]`).
            //   1. `[signer]` Candy Guard authority.
            //   2. `[writable]` Address to receive the funds (must match the `destination` address
            //                   of the guard configuration).
            //   3. `[]` System program account.
            FreezeInstruction::UnlockFunds => {
                msg!("FreezeSolPayment: unlock_funds");
                unlock_funds(ctx, data, freeze_guard.destination)
            }
        }
    }
}

impl Condition for FreezeSolPayment {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let candy_guard_key = &ctx.accounts.candy_guard.key();
        let candy_machine_key = &ctx.accounts.candy_machine.key();

        // validates the additional accounts

        let index = evaluation_context.account_cursor;
        let freeze_pda = Self::get_account_info(ctx, index)?;
        evaluation_context.account_cursor += 1;

        let seeds = [
            FreezeEscrow::PREFIX_SEED,
            self.destination.as_ref(),
            candy_guard_key.as_ref(),
            candy_machine_key.as_ref(),
        ];

        let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);
        assert_keys_equal(freeze_pda.key, &pda)?;

        if freeze_pda.data_is_empty() {
            return err!(CandyGuardError::FreezeNotInitialized);
        }

        let nft_ata = Self::get_account_info(ctx, index + 1)?;
        evaluation_context.account_cursor += 1;
        assert_is_ata(nft_ata, ctx.accounts.payer.key, ctx.accounts.nft_mint.key)?;

        evaluation_context
            .indices
            .insert("freeze_sol_payment", index);

        if ctx.accounts.payer.lamports() < self.lamports {
            msg!(
                "Require {} lamports, accounts has {} lamports",
                self.lamports,
                ctx.accounts.payer.lamports(),
            );
            return err!(CandyGuardError::NotEnoughSOL);
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let freeze_pda =
            Self::get_account_info(ctx, evaluation_context.indices["freeze_sol_payment"])?;

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.payer.key(),
                &freeze_pda.key(),
                self.lamports,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                freeze_pda.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    fn post_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // freezes the nft
        freeze_nft(
            ctx,
            evaluation_context.indices["freeze_sol_payment"],
            &self.destination,
        )
    }
}

/// PDA to track whether an address has been validated or not.
#[account]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct FreezeEscrow {
    pub candy_machine: Pubkey,
    pub allow_thaw: bool,
    pub frozen_count: u64,
    pub first_mint_time: Option<i64>,
    pub freeze_period: i64,
}

impl FreezeEscrow {
    /// Maximum account size.
    pub const SIZE: usize = 8 // discriminator
        + 32    // candy_machine
        + 1     // allow_thawn
        + 8     // frozen_count
        + 1 + 8 // option + first_mint_time
        + 8; // freeze time

    /// Prefix used as seed.
    pub const PREFIX_SEED: &'static [u8] = b"freeze_escrow";

    /// Maximum freeze period in seconds (30 days).
    pub const MAX_FREEZE_TIME: i64 = 60 * 60 * 24 * 30;

    pub fn init(
        &mut self,
        candy_machine: Pubkey,
        first_mint_time: Option<i64>,
        freeze_period: i64,
    ) {
        self.candy_machine = candy_machine;
        self.allow_thaw = false;
        self.frozen_count = 0;
        self.first_mint_time = first_mint_time;
        self.freeze_period = freeze_period;
    }

    pub fn is_thaw_allowed(&self, candy_machine: &CandyMachine, current_timestamp: i64) -> bool {
        if self.allow_thaw || candy_machine.items_redeemed >= candy_machine.data.items_available {
            return true;
        } else if let Some(first_mint_time) = self.first_mint_time {
            if current_timestamp >= first_mint_time + self.freeze_period {
                return true;
            }
        }

        false
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum FreezeInstruction {
    Initialize,
    Thaw,
    UnlockFunds,
}

/// Helper function to freeze an nft.
pub fn freeze_nft<'info>(
    ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
    account_index: usize,
    destination: &Pubkey,
) -> Result<()> {
    let freeze_pda = get_account_info(ctx, account_index)?;

    let mut freeze_escrow: Account<FreezeEscrow> = Account::try_from(freeze_pda)?;
    freeze_escrow.frozen_count += 1;

    if freeze_escrow.first_mint_time.is_none() {
        let clock = Clock::get()?;
        freeze_escrow.first_mint_time = Some(clock.unix_timestamp);
    }

    freeze_escrow.exit(&crate::ID)?;

    let candy_guard_key = &ctx.accounts.candy_guard.key();
    let candy_machine_key = &ctx.accounts.candy_machine.key();
    let payer = &ctx.accounts.payer;

    let seeds = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
    ];
    let (_, bump) = Pubkey::find_program_address(&seeds, &crate::ID);

    let signer = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
        &[bump],
    ];

    let nft_ata = get_account_info(ctx, account_index + 1)?;

    let mut freeze_ix = freeze_delegated_account(
        mpl_token_metadata::ID,
        freeze_pda.key(),
        nft_ata.key(),
        ctx.accounts.nft_master_edition.key(),
        ctx.accounts.nft_mint.key(),
    );

    freeze_ix.accounts[0] = AccountMeta::new_readonly(freeze_pda.key(), true);

    invoke(
        &approve(
            &spl_token::ID,
            &nft_ata.key(),
            &freeze_pda.key(),
            &payer.key(),
            &[],
            1,
        )?,
        &[
            nft_ata.to_account_info(),
            freeze_pda.to_account_info(),
            payer.to_account_info(),
        ],
    )?;
    invoke_signed(
        &freeze_ix,
        &[
            freeze_pda.to_account_info(),
            nft_ata.to_account_info(),
            ctx.accounts.nft_master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
        ],
        &[&signer],
    )?;

    Ok(())
}

/// Helper function to initialize the freeze pda.
pub fn initialize_freeze<'info>(
    ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
    data: Vec<u8>,
    destination: Pubkey,
) -> Result<()> {
    let candy_guard_key = &ctx.accounts.candy_guard.key();
    let candy_machine_key = &ctx.accounts.candy_machine.key();

    let seeds = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
    ];
    let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);

    let freeze_pda = get_account_info(ctx, 0)?;
    assert_keys_equal(freeze_pda.key, &pda)?;

    let authority = get_account_info(ctx, 1)?;

    if !(cmp_pubkeys(authority.key, &ctx.accounts.candy_guard.authority) && authority.is_signer) {
        return err!(CandyGuardError::MissingRequiredSignature);
    }

    if freeze_pda.data_is_empty() {
        // checking if we got the correct system_program
        let system_program = get_account_info(ctx, 2)?;
        assert_keys_equal(&system_program::ID, &system_program.key())?;

        let signer = [
            FreezeEscrow::PREFIX_SEED,
            destination.as_ref(),
            candy_guard_key.as_ref(),
            candy_machine_key.as_ref(),
            &[bump],
        ];
        let rent = Rent::get()?;

        invoke_signed(
            &system_instruction::create_account(
                &ctx.accounts.payer.key(),
                &pda,
                rent.minimum_balance(FreezeEscrow::SIZE),
                FreezeEscrow::SIZE as u64,
                &crate::ID,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                freeze_pda.to_account_info(),
            ],
            &[&signer],
        )?;
    } else {
        return err!(CandyGuardError::FreezeEscrowAlreadyExists);
    }

    // offset 1 to 9 (8 bytes) since the first byte is the freeze
    // instruction identifier
    let freeze_period = if let Ok(period) = i64::try_from_slice(&data[1..9]) {
        period
    } else {
        return err!(CandyGuardError::MissingFreezePeriod);
    };

    if freeze_period > FreezeEscrow::MAX_FREEZE_TIME {
        return err!(CandyGuardError::ExceededMaximumFreezePeriod);
    }

    // initilializes the escrow account (safe to be unchecked since the account
    // must be empty at this point)
    let mut freeze_escrow: Account<FreezeEscrow> = Account::try_from_unchecked(freeze_pda)?;
    freeze_escrow.init(*candy_machine_key, None, freeze_period);
    freeze_escrow.exit(&crate::ID)?;

    Ok(())
}

/// Helper function to thaw an nft.
pub fn thaw_nft<'info>(
    ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
    _data: Vec<u8>,
    destination: Pubkey,
) -> Result<()> {
    let candy_guard = &ctx.accounts.candy_guard;
    let candy_machine = &ctx.accounts.candy_machine;
    let current_timestamp = Clock::get()?.unix_timestamp;

    let freeze_pda = get_account_info(ctx, 0)?;
    let mut freeze_escrow: Account<FreezeEscrow> = Account::try_from(freeze_pda)?;

    if !(freeze_escrow.is_thaw_allowed(candy_machine, current_timestamp)
        || candy_machine.to_account_info().data_is_empty())
    {
        return err!(CandyGuardError::ThawNotEnabled);
    }

    let nft_mint = get_account_info(ctx, 1)?;
    let nft_owner = get_account_info(ctx, 2)?;

    let nft_ata = get_account_info(ctx, 3)?;
    let nft_token_account = TokenAccount::unpack(&nft_ata.try_borrow_data()?)?;

    assert_keys_equal(nft_mint.key, &nft_token_account.mint)?;
    assert_keys_equal(nft_owner.key, &nft_token_account.owner)?;

    let nft_master_edition = get_account_info(ctx, 4)?;
    let payer = &ctx.accounts.payer;

    let token_program = get_account_info(ctx, 5)?;
    let token_metadata_program = get_account_info(ctx, 6)?;

    let candy_guard_key = candy_guard.key();
    let candy_machine_key = candy_machine.key();

    let seeds = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
    ];
    let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);
    assert_keys_equal(&pda, freeze_pda.key)?;

    let signer = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
        &[bump],
    ];

    if nft_token_account.is_frozen() {
        invoke_signed(
            &thaw_delegated_account(
                mpl_token_metadata::ID,
                freeze_pda.key(),
                nft_ata.key(),
                nft_master_edition.key(),
                nft_mint.key(),
            ),
            &[
                freeze_pda.to_account_info(),
                nft_ata.to_account_info(),
                nft_master_edition.to_account_info(),
                nft_mint.to_account_info(),
                token_program.to_account_info(),
                token_metadata_program.to_account_info(),
            ],
            &[&signer],
        )?;
        freeze_escrow.frozen_count = freeze_escrow.frozen_count.saturating_sub(1);
    } else {
        msg!("Token account is not frozen");
    }

    if cmp_pubkeys(&payer.key(), &nft_owner.key()) {
        msg!("Revoking authority");
        invoke(
            &revoke(&spl_token::ID, &nft_ata.key(), &payer.key(), &[])?,
            &[nft_ata.to_account_info(), payer.to_account_info()],
        )?;
    } else {
        msg!("Token account owner is not signer, authority not revoked");
    }
    // save the account state
    freeze_escrow.exit(&crate::ID)?;

    Ok(())
}

/// Helper function to unlock funds.
fn unlock_funds<'info>(
    ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
    _data: Vec<u8>,
    destination: Pubkey,
) -> Result<()> {
    let candy_guard_key = &ctx.accounts.candy_guard.key();
    let candy_machine_key = &ctx.accounts.candy_machine.key();

    let freeze_pda = get_account_info(ctx, 0)?;
    let freeze_escrow: Account<FreezeEscrow> = Account::try_from(freeze_pda)?;

    let seeds = [
        FreezeEscrow::PREFIX_SEED,
        destination.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
    ];
    let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);
    assert_keys_equal(freeze_pda.key, &pda)?;

    // authority must the a signer
    let authority = get_account_info(ctx, 1)?;

    if !(cmp_pubkeys(authority.key, &ctx.accounts.candy_guard.authority) && authority.is_signer) {
        return err!(CandyGuardError::MissingRequiredSignature);
    }

    // all NFTs must be thaw
    if freeze_escrow.frozen_count > 0 {
        return err!(CandyGuardError::UnlockNotEnabled);
    }

    let destination_address = get_account_info(ctx, 2)?;
    // funds should go to the destination account
    assert_keys_equal(destination_address.key, &destination)?;

    freeze_escrow.close(destination_address.to_account_info())?;

    Ok(())
}
