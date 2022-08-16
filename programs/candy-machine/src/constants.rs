pub use mpl_token_metadata::state::{
    MAX_CREATOR_LEN, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
};
use solana_program::pubkey::Pubkey;

// Empty value used for string padding.
pub const NULL_STRING: &str = "\0";

// Constant to define the replacement index string.
pub const REPLACEMENT_INDEX: &str = "$ID$";

// Constant to define the replacement index increment string.
pub const REPLACEMENT_INDEX_INCREMENT: &str = "$ID+1$";

// Empty string constant.
pub const EMPTY_STR: &str = "";

// Determine the start of the account hidden section.
pub const HIDDEN_SECTION: usize = 8           // discriminator
    + 8                                       // features
    + 32                                      // base
    + 32                                      // wallet
    + 32                                      // authority
    + 33                                      // (optional) token mint
    + 8                                       // items redeemed
    + 8                                       // items available
    + 1                                       // bump
    + 4 + MAX_SYMBOL_LENGTH                   // u32 len + symbol
    + 2                                       // seller fee basis points
    + 8                                       // max supply
    + 1                                       // is mutable
    + 1                                       // retain authority
    + 4 + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN // u32 len + creators vec
    + 4 + MAX_NAME_LENGTH                     // u32 len +prefix_name length (config line setting)
    + 4                                       // name length
    + 4 + MAX_URI_LENGTH                      // u32 len +prefix_uri length
    + 4                                       // uri length
    + 1                                       // option (hidden setting)
    + 4 + MAX_NAME_LENGTH                     // u32 len +name length
    + 4 + MAX_URI_LENGTH                      // u32 len +uri length
    + 32; // hash

//--

pub const EXPIRE_OFFSET: i64 = 10 * 60;
pub const BLOCK_HASHES: Pubkey =
    solana_program::pubkey!("SysvarRecentB1ockHashes11111111111111111111");
pub const BOT_FEE: u64 = 10000000;
pub const PREFIX: &str = "candy_machine";
pub const COLLECTIONS_FEATURE_INDEX: usize = 0;
pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
pub const COLLECTION_PDA_SIZE: usize = 8 + 64;
pub const GUMDROP_ID: Pubkey =
    solana_program::pubkey!("gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a");
pub const CUPCAKE_ID: Pubkey =
    solana_program::pubkey!("cakeGJxEdGpZ3MJP8sM3QypwzuzZpko1ueonUQgKLPE");
pub const A_TOKEN: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
