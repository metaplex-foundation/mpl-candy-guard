use anchor_lang::{prelude::*, AnchorDeserialize};
use solana_program::program_memory::sol_memcmp;

use crate::{errors::CandyGuardError, guards::*};
use mpl_candy_guard_derive::GuardSet;

// Bytes offset for the start of the data section:
//     8 (discriminator)
//  + 32 (base)
//  +  1 (bump)
//  + 32 (authority)
pub const DATA_OFFSET: usize = 8 + 32 + 1 + 32;

// Maximim group label size.
pub const MAX_LABEL_SIZE: usize = 6;

// Seed value for PDA.
pub const SEED: &[u8] = b"candy_guard";

#[account]
#[derive(Default)]
pub struct CandyGuard {
    // Base key used to generate the PDA
    pub base: Pubkey,
    // Bump seed
    pub bump: u8,
    // Authority of the guard
    pub authority: Pubkey,
    // after this there is a flexible amount of data to serialize
    // data (CandyGuardData struct) of the available guards; the size
    // of the data is adjustable as new guards are implemented (the
    // account is resized using realloc)
    //
    // available guards:
    //  1) bot tax
    //  2) lamports
    //  3) spl token
    //  4) live date
    //  5) third party signer
    //  6) whitelist
    //  7) gatekeeper
    //  8) end settings
    //  9) allow list
    // 10) mint limit
    // 11) nft payment
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CandyGuardData {
    pub default: GuardSet,
    pub groups: Option<Vec<Group>>,
}

/// A group represent a specific set of guards. When groups are used, transactions
/// must specify which group should be used during validation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Group {
    pub label: String,
    pub guards: GuardSet,
}

/// The set of guards available.
#[derive(GuardSet, AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GuardSet {
    /// Last instruction check and bot tax (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Lamports guard (set the price for the mint in lamports).
    pub lamports: Option<Lamports>,
    /// Spl-token guard (set the price for the mint in spl-token amount).
    pub spl_token: Option<SplToken>,
    /// Live data guard (controls when minting is allowed).
    pub live_date: Option<LiveDate>,
    /// Third party signer guard.
    pub third_party_signer: Option<ThirdPartySigner>,
    /// Whitelist guard (whitelist mint settings).
    pub whitelist: Option<Whitelist>,
    /// Gatekeeper guard
    pub gatekeeper: Option<Gatekeeper>,
    /// End settings guard
    pub end_settings: Option<EndSettings>,
    /// Allow list guard
    pub allow_list: Option<AllowList>,
    /// Mint limit guard
    pub mint_limit: Option<MintLimit>,
    /// NFT Payment
    pub nft_payment: Option<NftPayment>,
}

impl CandyGuardData {
    /// Serialize the candy guard data into the specified data array.
    pub fn save(&self, data: &mut [u8]) -> Result<()> {
        let mut cursor = 0;

        // saves the 'default' guard set
        let _ = self.default.to_data(data)?;
        cursor += self.default.size();

        // stores the number of 'groups' guard set
        let group_counter = if let Some(groups) = &self.groups {
            groups.len() as u32
        } else {
            0
        };
        data[cursor..cursor + 4].copy_from_slice(&u32::to_le_bytes(group_counter));
        cursor += 4;

        // saves each individual 'groups' guard set
        if let Some(groups) = &self.groups {
            for group in groups {
                // label
                if group.label.len() > MAX_LABEL_SIZE {
                    return err!(CandyGuardError::LabelExceededLength);
                }
                data[cursor..cursor + group.label.len()].copy_from_slice(group.label.as_bytes());
                cursor += MAX_LABEL_SIZE;
                // guard set
                let _ = group.guards.to_data(&mut data[cursor..])?;
                cursor += group.guards.size();
            }
        }

        Ok(())
    }

    /// Deserializes the guards. Only attempts the deserialization of individuals guards
    /// if the data slice is large enough.
    pub fn load(data: &[u8]) -> Result<Self> {
        let (default, _) = GuardSet::from_data(data)?;
        let mut cursor = default.size();

        let group_counter = u32::from_le_bytes(*arrayref::array_ref![data, cursor, 4]);
        cursor += 4;

        let groups = if group_counter > 0 {
            let mut groups = Vec::with_capacity(group_counter as usize);

            for _i in 0..group_counter {
                let label = String::try_from_slice(&data[cursor..])?;
                cursor += MAX_LABEL_SIZE;
                let (guards, _) = GuardSet::from_data(&data[cursor..])?;
                cursor += guards.size();
                groups.push(Group { label, guards });
            }

            Some(groups)
        } else {
            None
        };

        Ok(Self { default, groups })
    }

    pub fn active_set(data: &[u8], label: Option<String>) -> Result<GuardSet> {
        // root guard set
        let (mut root, _) = GuardSet::from_data(data)?;
        let mut cursor = root.size();

        // number of groups
        let group_counter = u32::from_le_bytes(*arrayref::array_ref![data, cursor, 4]);
        cursor += 4;

        if group_counter > 0 {
            if let Some(label) = label {
                let label_slice = label.as_bytes();
                // retrieves the selected gorup
                for _i in 0..group_counter {
                    if sol_memcmp(label_slice, &data[cursor..], label_slice.len()) == 0 {
                        cursor += MAX_LABEL_SIZE;
                        let (guards, _) = GuardSet::from_data(&data[cursor..])?;
                        root.merge(guards);
                        // we found our group
                        return Ok(root);
                    } else {
                        cursor += MAX_LABEL_SIZE;
                        let features = u64::from_le_bytes(*arrayref::array_ref![data, cursor, 8]);
                        cursor += GuardSet::bytes_count(features);
                    }
                }
                return err!(CandyGuardError::GroupNotFound);
            }
            // if we have groups, label is required
            return err!(CandyGuardError::RequiredGroupLabelNotFound);
        } else if label.is_some() {
            return err!(CandyGuardError::GroupNotFound);
        }

        Ok(root)
    }

    pub fn size(&self) -> usize {
        let mut size = DATA_OFFSET + self.default.size();
        size += 4; // u32 (number of groups)

        if let Some(groups) = &self.groups {
            size += groups
                .iter()
                .map(|group| MAX_LABEL_SIZE + group.guards.size())
                .sum::<usize>();
        }

        size
    }
}
