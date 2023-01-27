use mpl_hydra::{state::{Fanout}, processors::AddMemberArgs};
use std::{ slice::Iter};


use  mpl_hydra::cpi::accounts::AddMemberWithNFT;

use super::{
    *,
};
use crate::{ state::GuardType};


/// Guard is used to:
/// * create an nft model hydra on cm creation
/// * assign nfts to hydra on mint
/// * ????
/// * profit
///
/// hail hydra
/// 

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Hydra {
    pub shares: u64,
    pub fanout: Fanout
}

impl Guard for Hydra {
    fn size() -> usize {
        8 + 1 // u64 + bool
    }

    fn mask() -> u64 {
        GuardType::as_mask(GuardType::Hydra)
    }
}

impl Condition for Hydra {
    fn validate<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {

        Ok(())
    }
}

impl Hydra {
    pub fn add_nft<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,

        remaining_accounts: &mut Iter<AccountInfo<'info>>,
    ) -> Result<()> {
        let fanout = &next_account_info(remaining_accounts)?;
        let system_program = ctx.accounts.system_program.to_account_info();
        let membership_account = next_account_info(remaining_accounts)?;
        let mint = next_account_info(remaining_accounts)?;
        let metadata = next_account_info(remaining_accounts)?;
        let rent = next_account_info(remaining_accounts)?;
        let token_program = ctx.accounts.token_program.to_account_info();
        let hydra_program = next_account_info(remaining_accounts)?;

        let add_ix = AddMemberWithNFT {
            authority: ctx.accounts.payer.to_account_info(),
            fanout: fanout.to_account_info(),
            membership_account: membership_account.to_account_info(),
            mint: mint.to_account_info(),
            metadata: metadata.to_account_info(),
            system_program: system_program,
            rent: rent.to_account_info(),
            token_program: token_program
        };
        let args = AddMemberArgs {
            shares: 100
        };
        let cpi_ctx = CpiContext::new(hydra_program.to_account_info(), add_ix);

        mpl_hydra::cpi::process_add_member_nft(cpi_ctx, args)?;
        msg!(
            "Added nft",
        );

        Ok(())
    }
}
