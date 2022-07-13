use anchor_lang::error_code;

#[error_code]
pub enum CandyGuardError {
    #[msg("Mint is not live")]
    MintNotLive,
    #[msg("Could not deserialize guard")]
    DeserializationError,
}