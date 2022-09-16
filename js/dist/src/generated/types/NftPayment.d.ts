import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
export declare type NftPayment = {
    burn: boolean;
    requiredCollection: web3.PublicKey;
};
export declare const nftPaymentBeet: beet.BeetArgsStruct<NftPayment>;
