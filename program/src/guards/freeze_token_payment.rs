use super::{freeze_sol_payment::freeze_nft, *};

use anchor_lang::AccountsClose;
use solana_program::program::{invoke, invoke_signed};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::instruction::close_account;

use crate::{
    errors::CandyGuardError,
    guards::freeze_sol_payment::{initialize_freeze, thaw_nft},
    utils::{
        assert_is_ata, assert_keys_equal, cmp_pubkeys, spl_token_transfer, TokenTransferParams,
    },
};

/// Guard that charges an amount in a specified spl-token as payment for the mint with a freeze period.
///
/// List of accounts required:
///
///   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
///           destination_ata pubkey, candy guard pubkey, candy machine pubkey]`).
///   1. `[]` Associate token account of the NFT (seeds `[payer pubkey, nft mint pubkey]`).
///   2. `[writable]` Token account holding the required amount.
///   3. `[writable]` Associate token account of the Freeze PDA (seeds `[freeze PDA pubkey, nft mint pubkey]`).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FreezeTokenPayment {
    pub amount: u64,
    pub mint: Pubkey,
    pub destination_ata: Pubkey,
}

impl Guard for FreezeTokenPayment {
    fn size() -> usize {
        8    // amount
        + 32 // token mint
        + 32 // destination ata
    }

    fn mask() -> u64 {
        0b1u64 << 17
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

        let freeze_guard = if let Some(freeze_guard) = &guard_set.freeze_token_payment {
            freeze_guard
        } else {
            return err!(CandyGuardError::FreezeGuardNotEnabled);
        };

        match instruction {
            // List of accounts required:
            //
            //   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_escrow",
            //                   destination pubkey, candy guard pubkey, candy machine pubkey]`).
            //   1. `[signer]` Candy Guard authority.
            //   2. `[]` System program account.
            //   3. `[writable]` Associate token account of the Freeze PDA (seeds `[freeze PDA pubkey, nft mint pubkey]`).
            //   4. `[]` Token mint account.
            FreezeInstruction::Initialize => {
                msg!("FreezeTokenPayment: initialize");
                // initializes the freeze pda (the check of the authority as signer is done
                // during the initialization)
                initialize_freeze(ctx, data, freeze_guard.destination_ata)?;

                // initializes the freeze ata

                let freeze_pda = get_account_info(ctx, 0)?;
                let system_program = get_account_info(ctx, 2)?;
                let freeze_ata = get_account_info(ctx, 3)?;
                let token_mint = get_account_info(ctx, 4)?;

                assert_keys_equal(
                    &get_associated_token_address(freeze_pda.key, token_mint.key),
                    freeze_ata.key,
                )?;

                invoke(
                    &create_associated_token_account(
                        ctx.accounts.payer.key,
                        freeze_pda.key,
                        token_mint.key,
                    ),
                    &[
                        ctx.accounts.payer.to_account_info(),
                        freeze_ata.to_account_info(),
                        freeze_pda.to_account_info(),
                        token_mint.to_account_info(),
                        system_program.to_account_info(),
                    ],
                )?;

                Ok(())
            }
            FreezeInstruction::Thaw => {
                msg!("FreezeTokenPayment: thaw");
                thaw_nft(ctx, data, freeze_guard.destination_ata)
            }

            FreezeInstruction::UnlockFunds => {
                msg!("FreezeTokenPayment: unlock_funds");
                unlock_funds(ctx, data, freeze_guard.mint, freeze_guard.destination_ata)
            }
        }
    }
}

impl Condition for FreezeTokenPayment {
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
            self.destination_ata.as_ref(),
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

        let token_account_info = Self::get_account_info(ctx, index + 2)?;
        // validate freeze_pda ata
        let destination_ata = Self::get_account_info(ctx, index + 3)?;
        assert_is_ata(destination_ata, &freeze_pda.key(), &self.mint)?;

        evaluation_context.account_cursor += 2;

        let token_account =
            assert_is_ata(token_account_info, &ctx.accounts.payer.key(), &self.mint)?;

        if token_account.amount < self.amount {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        evaluation_context
            .indices
            .insert("freeze_token_payment", index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.indices["freeze_token_payment"];
        // the accounts have already been validated
        let token_account_info = Self::get_account_info(ctx, index + 2)?;
        let destination_ata = Self::get_account_info(ctx, index + 3)?;

        spl_token_transfer(TokenTransferParams {
            source: token_account_info.to_account_info(),
            destination: destination_ata.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: self.amount,
        })?;

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
            evaluation_context.indices["freeze_token_payment"],
            &self.destination_ata,
        )
    }
}

// Unlocks frozen funds.
//
// List of accounts required:
//
//   0. `[writable]` Freeze PDA to receive the funds (seeds `["freeze_sol_payment",
//                   destination pubkey, candy guard pubkey, candy machine pubkey]`).
//   1. `[signer]` Candy Guard authority.
//   2. `[writable]` Associate token account of the Freeze PDA (seeds `[freeze PDA pubkey, nft mint pubkey]`).
//   3. `[writable]` Address to receive the funds (must match the `destination_ata` address
//                   of the guard configuration).
//   4. `[]` Token program account.
//   5. `[]` System program account.
fn unlock_funds<'info>(
    ctx: &Context<'_, '_, '_, 'info, Route<'info>>,
    _data: Vec<u8>,
    mint: Pubkey,
    destination_ata: Pubkey,
) -> Result<()> {
    let candy_guard_key = &ctx.accounts.candy_guard.key();
    let candy_machine_key = &ctx.accounts.candy_machine.key();

    let freeze_pda = get_account_info(ctx, 0)?;
    let freeze_escrow: Account<FreezeEscrow> = Account::try_from(freeze_pda)?;

    let seeds = [
        FreezeEscrow::PREFIX_SEED,
        destination_ata.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
    ];
    let (pda, bump) = Pubkey::find_program_address(&seeds, &crate::ID);
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

    let freeze_ata = get_account_info(ctx, 2)?;
    let source_ata = assert_is_ata(freeze_ata, &freeze_pda.key(), &mint)?;

    let destination_ata_account = get_account_info(ctx, 3)?;
    assert_keys_equal(&destination_ata, destination_ata_account.key)?;

    let token_program = get_account_info(ctx, 4)?;
    assert_keys_equal(token_program.key, &Token::id())?;

    // transfer the tokens

    let signer = [
        FreezeEscrow::PREFIX_SEED,
        destination_ata.as_ref(),
        candy_guard_key.as_ref(),
        candy_machine_key.as_ref(),
        &[bump],
    ];

    spl_token_transfer(TokenTransferParams {
        source: freeze_ata.to_account_info(),
        destination: destination_ata_account.to_account_info(),
        authority: freeze_pda.to_account_info(),
        authority_signer_seeds: &signer,
        token_program: token_program.to_account_info(),
        amount: source_ata.amount,
    })?;

    // close the freeze ata

    invoke_signed(
        &close_account(
            token_program.key,
            freeze_ata.key,
            authority.key,
            freeze_pda.key,
            &[],
        )?,
        &[
            freeze_ata.to_account_info(),
            authority.to_account_info(),
            freeze_pda.to_account_info(),
            token_program.to_account_info(),
        ],
        &[&signer],
    )?;

    // the rent for the freeze escrow goes back to the authority
    freeze_escrow.close(authority.to_account_info())?;

    Ok(())
}
