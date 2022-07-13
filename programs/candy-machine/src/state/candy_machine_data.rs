use anchor_lang::prelude::*;

/// Candy machine configuration data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct CandyMachineData {
    /// Price of an asset
    pub price: u64,
    /// Number of assets available
    pub items_available: u64,
    /// Symbol for the asset
    pub symbol: String,
    /// Secondary sales royalty basis points (0-10000)
    pub seller_fee_basis_points: u16,
    /// Max supply of each individual asset (default 0)
    pub max_supply: u64,
    /// Indicates if the asset is mutable or not
    pub is_mutable: bool,
    /// Indicates whether to retain the update authority or not
    pub retain_authority: bool,
    /// List of creators
    pub creators: Vec<Creator>,
    /// Config line settings
    pub config_lines_settings: ConfigListSettings,
    /// Hidden setttings
    pub hidden_settings: Option<HiddenSettings>,
}

// Creator information.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    /// Pubkey address
    pub address: Pubkey,
    /// Whether the creator is verified or not
    pub verified: bool,
    // Share of secondary sales royalty
    pub percentage_share: u8,
}

/// Hidden settings for large mints used with off-chain data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct HiddenSettings {
    /// Asset prefix name
    pub name: String,
    /// Shared URI
    pub uri: String,
    /// Hash of the hidden settings file
    pub hash: [u8; 32],
}

/// Config line settings to allocate space for individual name + URI.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct ConfigListSettings {
    /// Common name prefix
    pub prefix_name: String,
    /// Length of the remaining part of the name
    pub name_length: u32,
    /// Common URI prefix
    pub prefix_uri: String,
    /// Length of the remaining part of the URI
    pub uri_length: u32,
}
