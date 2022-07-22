use anchor_lang::error_code;

#[error_code]
pub enum CandyGuardError {
    #[msg("Could not deserialize guard")]
    DeserializationError,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("Account does not have correct owner!")]
    IncorrectOwner,
    #[msg("Account is not initialized!")]
    Uninitialized,
    // live date
    #[msg("Mint is not live")]
    MintNotLive,
    // Whitelist
    #[msg("Token burn failed")]
    TokenBurnFailed,
    #[msg("No whitelist token present")]
    NoWhitelistToken,
}
