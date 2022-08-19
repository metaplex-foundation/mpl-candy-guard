use anchor_lang::prelude::*;

pub use errors::CandyError;
pub use state::*;
pub use utils::*;

use instructions::*;
pub use state::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

declare_id!("AaciizBuB6XnuztWxVq52KJwDv8WggF8yAEhrRx2ECEU");

#[program]
pub mod candy_machine {

    use super::*;

    pub fn add_collection(ctx: Context<AddCollection>) -> Result<()> {
        instructions::add_collection(ctx)
    }

    pub fn add_config_lines(
        ctx: Context<AddConfigLines>,
        index: u32,
        config_lines: Vec<ConfigLine>,
    ) -> Result<()> {
        instructions::add_config_lines(ctx, index, config_lines)
    }

    pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
        instructions::initialize(ctx, data)
    }

    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
        creator_bump: u8,
    ) -> Result<()> {
        instructions::mint(ctx, creator_bump)
    }

    pub fn remove_collection(ctx: Context<RemoveCollection>) -> Result<()> {
        instructions::remove_collection(ctx)
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_authority(ctx, new_authority)
    }

    pub fn update(ctx: Context<Update>, data: CandyMachineData) -> Result<()> {
        instructions::update(ctx, data)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
