declare type ErrorWithCode = Error & {
    code: number;
};
declare type MaybeErrorWithCode = ErrorWithCode | null | undefined;
export declare class InvalidAccountSizeError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class DeserializationErrorError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class PublicKeyMismatchError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class DataIncrementLimitExceededError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class IncorrectOwnerError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class UninitializedError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingRemainingAccountError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NumericalOverflowErrorError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class RequiredGroupLabelNotFoundError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class GroupNotFoundError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class LabelExceededLengthError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CollectionKeyMismatchError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingCollectionAccountsError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class CollectionUpdateAuthorityKeyMismatchError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MintNotLastTransactionError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MintNotLiveError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NotEnoughSOLError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class TokenTransferFailedError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NotEnoughTokensError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingRequiredSignatureError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class TokenBurnFailedError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class NoWhitelistTokenError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class GatewayTokenInvalidError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class AfterEndSettingsDateError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class AfterEndSettingsMintAmountError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class InvalidMintTimeError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class AddressNotFoundInAllowedListError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class MissingAllowedListProofError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class AllowedMintLimitReachedError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare class InvalidNFTCollectionPaymentError extends Error {
    readonly code: number;
    readonly name: string;
    constructor();
}
export declare function errorFromCode(code: number): MaybeErrorWithCode;
export declare function errorFromName(name: string): MaybeErrorWithCode;
export {};
