import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare type Lamports = {
    amount: beet.bignum;
    destination: web3.PublicKey;
};
export declare const lamportsBeet: beet.BeetArgsStruct<Lamports>;
