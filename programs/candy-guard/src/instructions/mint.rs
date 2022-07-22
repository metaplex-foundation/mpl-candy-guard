use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::token::Token;
use candy_machine::{self, CandyMachine};

use crate::guards::EvaluationContext;
use crate::state::{CandyGuard, CandyGuardData};
use crate::utils::cmp_pubkeys;

pub fn mint<'info>(ctx: Context<'_, '_, '_, 'info, Mint<'info>>, creator_bump: u8) -> Result<()> {
    let info = ctx.accounts.candy_guard.to_account_info();
    let mut data = info.data.borrow_mut();

    let candy_guard_data = CandyGuardData::from_data(ctx.accounts.candy_guard.features, &mut data)?;
    let conditions = candy_guard_data.enabled_conditions();
    // context for this transaction
    let mut evaluation_context = EvaluationContext {
        discount_price: 0,
        is_authority: cmp_pubkeys(
            &ctx.accounts.candy_guard.authority,
            &ctx.accounts.payer.key(),
        ),
        is_presale: false,
        remaining_account_counter: 0,
    };

    for condition in conditions {
        if let Err(error) = condition.evaluate(&ctx, &candy_guard_data, &mut evaluation_context) {
            return if let Some(bot_tax) = candy_guard_data.bot_tax {
                // apply bot tax using bot_tax.lamports as fee
                bot_tax.punish_bots(
                    error,
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.candy_machine.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                )?;

                Ok(())
            } else {
                Err(error)
            };
        }
    }

    // TODO: all guards are successful, forward the transaction to Candy Machine using the
    // price from the evaluation_context
    msg!(
        "EvaluationContext: is_presale={}, is_authority={}, discount_price={}",
        evaluation_context.is_presale,
        evaluation_context.is_authority,
        evaluation_context.discount_price
    );

    let candy_machine_program = ctx.accounts.candy_machine_program.to_account_info();
    // candy machine mint instruction accounts
    let mint_ix = candy_machine::cpi::accounts::Mint {
        candy_machine: ctx.accounts.candy_machine.to_account_info(),
        candy_machine_creator: ctx.accounts.candy_machine_creator.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        wallet: ctx.accounts.wallet.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        mint_authority: ctx.accounts.mint_authority.to_account_info(),
        update_authority: ctx.accounts.update_authority.to_account_info(),
        master_edition: ctx.accounts.master_edition.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        recent_slothashes: ctx.accounts.recent_slothashes.to_account_info(),
        instruction_sysvar_account: ctx.accounts.instruction_sysvar_account.to_account_info(),
    };
    // candy machine mint CPI
    if let Err(error) = candy_machine::cpi::mint(
        CpiContext::new(candy_machine_program, mint_ix),
        creator_bump,
    ) {
        return if let Some(bot_tax) = candy_guard_data.bot_tax {
            // apply bot tax using bot_tax.lamports as fee
            bot_tax.punish_bots(
                error,
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.candy_machine.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            )?;

            Ok(())
        } else {
            Err(error)
        };
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct Mint<'info> {
    #[account(mut, constraint = candy_guard.authority == candy_machine.authority)]
    pub candy_guard: Account<'info, CandyGuard>,
    /// CHECK: account constraints checked in account trait
    #[account(address = candy_machine::id())]
    pub candy_machine_program: AccountInfo<'info>,
    #[account(mut, has_one = wallet)]
    pub candy_machine: Box<Account<'info, CandyMachine>>,
    // seeds and bump are not validated by the candy guard, they will be validated
    // by the CPI'd candy machine mint call
    /// CHECK: account constraints checked in account trait
    #[account(mut)]
    pub candy_machine_creator: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: wallet can be any account and is not written to or read
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,
    // with the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub mint_authority: Signer<'info>,
    pub update_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    pub recent_slothashes: UncheckedAccount<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    // remaining accounts:
    // > only needed if candy machine has a gatekeeper
    // gateway_token
    // > only needed if candy machine has a gatekeeper and it has expire_on_use set to true:
    // gateway program
    // network_expire_feature
    // > only needed if candy machine has whitelist_mint_settings
    // whitelist_token_account
    // > only needed if candy machine has whitelist_mint_settings and mode is BurnEveryTime
    // whitelist_token_mint
    // whitelist_burn_authority
    // > only needed if candy machine has token mint
    // token_account_info
    // transfer_authority_info
}
