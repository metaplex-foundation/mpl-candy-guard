"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.mintCounterBeet = exports.MintCounter = exports.mintCounterDiscriminator = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
const beetSolana = __importStar(require("@metaplex-foundation/beet-solana"));
exports.mintCounterDiscriminator = [29, 59, 15, 69, 46, 22, 227, 173];
class MintCounter {
    constructor(count) {
        this.count = count;
    }
    static fromArgs(args) {
        return new MintCounter(args.count);
    }
    static fromAccountInfo(accountInfo, offset = 0) {
        return MintCounter.deserialize(accountInfo.data, offset);
    }
    static async fromAccountAddress(connection, address) {
        const accountInfo = await connection.getAccountInfo(address);
        if (accountInfo == null) {
            throw new Error(`Unable to find MintCounter account at ${address}`);
        }
        return MintCounter.fromAccountInfo(accountInfo, 0)[0];
    }
    static gpaBuilder(programId = new web3.PublicKey('grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ')) {
        return beetSolana.GpaBuilder.fromStruct(programId, exports.mintCounterBeet);
    }
    static deserialize(buf, offset = 0) {
        return exports.mintCounterBeet.deserialize(buf, offset);
    }
    serialize() {
        return exports.mintCounterBeet.serialize({
            accountDiscriminator: exports.mintCounterDiscriminator,
            ...this,
        });
    }
    static get byteSize() {
        return exports.mintCounterBeet.byteSize;
    }
    static async getMinimumBalanceForRentExemption(connection, commitment) {
        return connection.getMinimumBalanceForRentExemption(MintCounter.byteSize, commitment);
    }
    static hasCorrectByteSize(buf, offset = 0) {
        return buf.byteLength - offset === MintCounter.byteSize;
    }
    pretty() {
        return {
            count: this.count,
        };
    }
}
exports.MintCounter = MintCounter;
exports.mintCounterBeet = new beet.BeetStruct([
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['count', beet.u16],
], MintCounter.fromArgs, 'MintCounter');
//# sourceMappingURL=MintCounter.js.map