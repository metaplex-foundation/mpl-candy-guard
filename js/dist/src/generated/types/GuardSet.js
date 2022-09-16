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
exports.guardSetBeet = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const BotTax_1 = require("./BotTax");
const Lamports_1 = require("./Lamports");
const SplToken_1 = require("./SplToken");
const LiveDate_1 = require("./LiveDate");
const ThirdPartySigner_1 = require("./ThirdPartySigner");
const Whitelist_1 = require("./Whitelist");
const Gatekeeper_1 = require("./Gatekeeper");
const EndSettings_1 = require("./EndSettings");
const AllowList_1 = require("./AllowList");
const MintLimit_1 = require("./MintLimit");
const NftPayment_1 = require("./NftPayment");
exports.guardSetBeet = new beet.FixableBeetArgsStruct([
    ['botTax', beet.coption(BotTax_1.botTaxBeet)],
    ['lamports', beet.coption(Lamports_1.lamportsBeet)],
    ['splToken', beet.coption(SplToken_1.splTokenBeet)],
    ['liveDate', beet.coption(LiveDate_1.liveDateBeet)],
    ['thirdPartySigner', beet.coption(ThirdPartySigner_1.thirdPartySignerBeet)],
    ['whitelist', beet.coption(Whitelist_1.whitelistBeet)],
    ['gatekeeper', beet.coption(Gatekeeper_1.gatekeeperBeet)],
    ['endSettings', beet.coption(EndSettings_1.endSettingsBeet)],
    ['allowList', beet.coption(AllowList_1.allowListBeet)],
    ['mintLimit', beet.coption(MintLimit_1.mintLimitBeet)],
    ['nftPayment', beet.coption(NftPayment_1.nftPaymentBeet)],
], 'GuardSet');
//# sourceMappingURL=GuardSet.js.map