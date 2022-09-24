use anchor_lang::error_code;

#[error_code]
pub enum CandyGuardError {
    #[msg("Could not save guard to account")]
    InvalidAccountSize,
    #[msg("Could not deserialize guard")]
    DeserializationError,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("Missing expected remaining account")]
    DataIncrementLimitExceeded,
    #[msg("Account does not have correct owner")]
    IncorrectOwner,
    #[msg("Account is not initialized")]
    Uninitialized,
    #[msg("Missing expected remaining account")]
    MissingRemainingAccount,
    #[msg("Numerical overflow error")]
    NumericalOverflowError,
    #[msg("Missing required group label")]
    RequiredGroupLabelNotFound,
    #[msg("Group not found")]
    GroupNotFound,
    #[msg("Group not found")]
    LabelExceededLength,
    #[msg("Candy machine is empty")]
    CandyMachineEmpty,
    // collection
    #[msg("Collection public key mismatch")]
    CollectionKeyMismatch,
    #[msg("Missing collection accounts")]
    MissingCollectionAccounts,
    #[msg("Collection update authority public key mismatch")]
    CollectionUpdateAuthorityKeyMismatch,
    // bot tax
    #[msg("Mint must be the last instructions of the transaction")]
    MintNotLastTransaction,
    // live date
    #[msg("Mint is not live")]
    MintNotLive,
    // native price
    #[msg("Not enough SOL to pay for the mint")]
    NotEnoughSOL,
    // spl-token price
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    // third-party signer
    #[msg("A signature was required but not found")]
    MissingRequiredSignature,
    // token gate
    #[msg("Token burn failed")]
    TokenBurnFailed,
    #[msg("Missing token gate")]
    MissingTokenGate,
    // gatekeeper
    #[msg("Gateway token is not valid")]
    GatewayTokenInvalid,
    // end date
    #[msg("Current time is after the set end date")]
    AfterEndDate,
    // allow list
    #[msg("Current time is not within the allowed mint time")]
    InvalidMintTime,
    #[msg("Address not found on the allowed list")]
    AddressNotFoundInAllowedList,
    #[msg("Missing allowed list proof")]
    MissingAllowedListProof,
    #[msg("The maximum number of allowed mints was reached")]
    AllowedMintLimitReached,
    #[msg("Invalid NFT Collection Payment")]
    InvalidNFTCollectionPayment,
    // redeemed amount
    #[msg("Current redemeed items is at the set maximum amount")]
    MaximumRedeemedAmount,
    // authority only
    #[msg("Address not authorized")]
    AddressNotAuthorized,
}
