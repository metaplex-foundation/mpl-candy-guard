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
exports.parseData = void 0;
const types_1 = require("./generated/types");
const bn_js_1 = require("bn.js");
const beet = __importStar(require("@metaplex-foundation/beet"));
const log_1 = require("./utils/log");
const MintLimit_1 = require("./generated/types/MintLimit");
const NftPayment_1 = require("./generated/types/NftPayment");
const GUARDS_SIZE = {
    botTax: 9,
    lamports: 40,
    splToken: 72,
    liveDate: 9,
    thirdPartySigner: 32,
    whitelist: 43,
    gatekeeper: 33,
    endSettings: 9,
    allowList: 32,
    mintLimit: 3,
    nftPayment: 33,
};
const GUARDS_COUNT = 11;
const MAX_LABEL_LENGTH = 6;
function determineGuards(buffer) {
    const enabled = new bn_js_1.BN(beet.u64.read(buffer, 0)).toNumber();
    const guards = [];
    for (let i = 0; i < GUARDS_COUNT; i++) {
        guards.push(!!((1 << i) & enabled));
    }
    const [botTaxEnabled, lamportsEnabled, splTokenEnabled, liveDateEnabled, thirdPartySignerEnabled, whitelistEnabled, gatekeeperEnabled, endSettingsEnabled, allowListEnabled, mintLimitEnabled, nftPaymentEnabled,] = guards;
    return {
        botTaxEnabled,
        lamportsEnabled,
        splTokenEnabled,
        liveDateEnabled,
        thirdPartySignerEnabled,
        whitelistEnabled,
        gatekeeperEnabled,
        endSettingsEnabled,
        allowListEnabled,
        mintLimitEnabled,
        nftPaymentEnabled,
    };
}
function parseData(buffer) {
    const { guardSet: defaultSet, offset } = parseGuardSet(buffer);
    const groupsCount = new bn_js_1.BN(beet.u32.read(buffer, offset)).toNumber();
    const groups = [];
    let cursor = beet.u32.byteSize + offset;
    for (let i = 0; i < groupsCount; i++) {
        const label = buffer.subarray(cursor, cursor + MAX_LABEL_LENGTH).toString();
        cursor += MAX_LABEL_LENGTH;
        const { guardSet: guards, offset } = parseGuardSet(buffer.subarray(cursor));
        groups.push({ label, guards });
        cursor += offset;
    }
    return {
        default: defaultSet,
        groups: groups.length === 0 ? null : groups,
    };
}
exports.parseData = parseData;
function parseGuardSet(buffer) {
    var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l;
    const guards = determineGuards(buffer);
    const { botTaxEnabled, liveDateEnabled, lamportsEnabled, splTokenEnabled, thirdPartySignerEnabled, whitelistEnabled, gatekeeperEnabled, endSettingsEnabled, allowListEnabled, mintLimitEnabled, nftPaymentEnabled, } = guards;
    (0, log_1.logDebug)('Guards: %O', guards);
    let cursor = beet.u64.byteSize;
    const data = {};
    if (botTaxEnabled) {
        const [botTax] = types_1.botTaxBeet.deserialize(buffer, cursor);
        data.botTax = botTax;
        cursor += GUARDS_SIZE.botTax;
    }
    if (lamportsEnabled) {
        const [lamports] = types_1.lamportsBeet.deserialize(buffer, cursor);
        data.lamports = lamports;
        cursor += GUARDS_SIZE.lamports;
    }
    if (splTokenEnabled) {
        const [splToken] = types_1.splTokenBeet.deserialize(buffer, cursor);
        data.splToken = splToken;
        cursor += GUARDS_SIZE.splToken;
    }
    if (liveDateEnabled) {
        const [liveDate] = types_1.liveDateBeet.deserialize(buffer, cursor);
        data.liveDate = liveDate;
        cursor += GUARDS_SIZE.liveDate;
    }
    if (thirdPartySignerEnabled) {
        const [thirdPartySigner] = types_1.thirdPartySignerBeet.deserialize(buffer, cursor);
        data.thirdPartySigner = thirdPartySigner;
        cursor += GUARDS_SIZE.thirdPartySigner;
    }
    if (whitelistEnabled) {
        const [whitelist] = types_1.whitelistBeet.deserialize(buffer, cursor);
        data.whitelist = whitelist;
        cursor += GUARDS_SIZE.whitelist;
    }
    if (gatekeeperEnabled) {
        const [gatekeeper] = types_1.gatekeeperBeet.deserialize(buffer, cursor);
        data.gatekeeper = gatekeeper;
        cursor += GUARDS_SIZE.gatekeeper;
    }
    if (endSettingsEnabled) {
        const [endSettings] = types_1.endSettingsBeet.deserialize(buffer, cursor);
        data.endSettings = endSettings;
        cursor += GUARDS_SIZE.endSettings;
    }
    if (allowListEnabled) {
        const [allowList] = types_1.allowListBeet.deserialize(buffer, cursor);
        data.allowList = allowList;
        cursor += GUARDS_SIZE.allowList;
    }
    if (mintLimitEnabled) {
        const [mintLimit] = MintLimit_1.mintLimitBeet.deserialize(buffer, cursor);
        data.mintLimit = mintLimit;
        cursor += GUARDS_SIZE.mintLimit;
    }
    if (nftPaymentEnabled) {
        const [nftPayment] = NftPayment_1.nftPaymentBeet.deserialize(buffer, cursor);
        data.nftPayment = nftPayment;
        cursor += GUARDS_SIZE.nftPayment;
    }
    return {
        guardSet: {
            botTax: (_a = data.botTax) !== null && _a !== void 0 ? _a : null,
            liveDate: (_b = data.liveDate) !== null && _b !== void 0 ? _b : null,
            lamports: (_c = data.lamports) !== null && _c !== void 0 ? _c : null,
            splToken: (_d = data.splToken) !== null && _d !== void 0 ? _d : null,
            thirdPartySigner: (_e = data.thirdPartySigner) !== null && _e !== void 0 ? _e : null,
            whitelist: (_f = data.whitelist) !== null && _f !== void 0 ? _f : null,
            gatekeeper: (_g = data.gateKeeper) !== null && _g !== void 0 ? _g : null,
            endSettings: (_h = data.endSettings) !== null && _h !== void 0 ? _h : null,
            allowList: (_j = data.allowList) !== null && _j !== void 0 ? _j : null,
            mintLimit: (_k = data.mintLimit) !== null && _k !== void 0 ? _k : null,
            nftPayment: (_l = data.nftPayment) !== null && _l !== void 0 ? _l : null,
        },
        offset: cursor,
    };
}
//# sourceMappingURL=parser.js.map