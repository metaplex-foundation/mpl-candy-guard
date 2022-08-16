use anchor_lang::error_code;

#[error_code]
pub enum CandyGuardError {
    #[msg("Could not deserialize guard")]
    DeserializationError,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("Account does not have correct owner")]
    IncorrectOwner,
    #[msg("Account is not initialized")]
    Uninitialized,
    // live date
    #[msg("Mint is not live")]
    MintNotLive,
    // whitelist
    #[msg("Token burn failed")]
    TokenBurnFailed,
    #[msg("No whitelist token present")]
    NoWhitelistToken,
    // native price
    #[msg("Not enough SOL to pay for the mint")]
    NotEnoughSOL,
    // spl-token price
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    #[msg("Mint must be the last instructions of the transaction")]
    MintNotLastTransaction,
    #[msg("Missing set collection during mint IX")]
    MissingCollectionInstruction,
}
