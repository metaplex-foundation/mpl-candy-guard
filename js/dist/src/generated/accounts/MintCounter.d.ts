/// <reference types="node" />
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export declare type MintCounterArgs = {
    count: number;
};
export declare const mintCounterDiscriminator: number[];
export declare class MintCounter implements MintCounterArgs {
    readonly count: number;
    private constructor();
    static fromArgs(args: MintCounterArgs): MintCounter;
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [MintCounter, number];
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey): Promise<MintCounter>;
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<{
        accountDiscriminator: any;
        count: any;
    }>;
    static deserialize(buf: Buffer, offset?: number): [MintCounter, number];
    serialize(): [Buffer, number];
    static get byteSize(): number;
    static getMinimumBalanceForRentExemption(connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    static hasCorrectByteSize(buf: Buffer, offset?: number): boolean;
    pretty(): {
        count: number;
    };
}
export declare const mintCounterBeet: beet.BeetStruct<MintCounter, MintCounterArgs & {
    accountDiscriminator: number[];
}>;
