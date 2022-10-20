import { BN } from 'bn.js';
import * as beet from '@metaplex-foundation/beet';
import { logDebug } from './utils/log';
import {
  allowListBeet,
  botTaxBeet,
  CandyGuardData,
  freezeSolPaymentBeet,
  freezeTokenPaymentBeet,
  gatekeeperBeet,
  Group,
  GuardSet,
  mintLimitBeet,
  nftPaymentBeet,
  startDateBeet,
  thirdPartySignerBeet,
  tokenGateBeet,
} from './generated';
import { solPaymentBeet } from './generated/types/SolPayment';
import { tokenPaymentBeet } from './generated/types/TokenPayment';
import { endDateBeet } from './generated/types/EndDate';
import { redeemedAmountBeet } from './generated/types/RedeemedAmount';
import { addressGateBeet } from './generated/types/AddressGate';
import { nftGateBeet } from './generated/types/NftGate';
import { nftBurnBeet } from './generated/types/NftBurn';
import { tokenBurnBeet } from './generated/types/TokenBurn';

/**
 * Matching the guards of the related struct in the Rust program.
 * Make sure to update this whenever the Rust struct changes.
 * ```
 * pub struct GuardSet {
 *   /// Last instruction check and bot tax (penalty for invalid transactions).
 *   pub bot_tax: Option<BotTax>,
 *   /// Sol payment guard (set the price for the mint in lamports).
 *   pub sol_payment: Option<SolPayment>,
 *   /// Token payment guard (set the price for the mint in spl-token amount).
 *   pub token_payment: Option<TokenPayment>,
 *   /// Start data guard (controls when minting is allowed).
 *   pub start_date: Option<StartDate>,
 *   /// Third party signer guard (requires an extra signer for the transaction).
 *   pub third_party_signer: Option<ThirdPartySigner>,
 *   /// Token gate guard (restrict access to holders of a specific token).
 *   pub token_gate: Option<TokenGate>,
 *   /// Gatekeeper guard (captcha challenge).
 *   pub gatekeeper: Option<Gatekeeper>,
 *   /// End date guard (set an end date to stop the mint).
 *   pub end_date: Option<EndDate>,
 *   /// Allow list guard (curated list of allowed addresses).
 *   pub allow_list: Option<AllowList>,
 *   /// Mint limit guard (add a limit on the number of mints per wallet).
 *   pub mint_limit: Option<MintLimit>,
 *   /// NFT Payment (charge an NFT in order to mint).
 *   pub nft_payment: Option<NftPayment>,
 *   /// Redeemed amount guard (add a limit on the overall number of items minted).
 *   pub redeemed_amount: Option<RedeemedAmount>,
 *   /// Address gate (check access against a specified address).
 *   pub address_gate: Option<AddressGate>,
 *   /// NFT gate guard (check access based on holding a specified NFT).
 *   pub nft_gate: Option<NftGate>,
 *   /// NFT burn guard (burn a specified NFT).
 *   pub nft_burn: Option<NftBurn>,
 *   /// Token burn guard (burn a specified amount of spl-token).
 *   pub token_burn: Option<TokenBurn>,
 *   /// Freeze sol payment (set the price for the mint in lamports with a freeze period).
 *   pub freeze_sol_payment: Option<FreezeSolPayment>,
 *   /// Freeze token payment guard (set the price for the mint in spl-token amount with a freeze period).
 *   pub freeze_token_payment: Option<FreezeTokenPayment>,
 * }
 * ```
 */

type Guards = {
  /* 01 */ botTaxEnabled: boolean;
  /* 02 */ solPaymentEnabled: boolean;
  /* 03 */ tokenPaymentEnabled: boolean;
  /* 04 */ startDateEnabled: boolean;
  /* 05 */ thirdPartySignerEnabled: boolean;
  /* 06 */ tokenGateEnabled: boolean;
  /* 07 */ gatekeeperEnabled: boolean;
  /* 08 */ endDateEnabled: boolean;
  /* 09 */ allowListEnabled: boolean;
  /* 10 */ mintLimitEnabled: boolean;
  /* 11 */ nftPaymentEnabled: boolean;
  /* 12 */ redeemedAmountEnabled: boolean;
  /* 13 */ addressGateEnabled: boolean;
  /* 14 */ nftGateEnabled: boolean;
  /* 15 */ nftBurnEnabled: boolean;
  /* 16 */ tokenBurnEnabled: boolean;
  /* 17 */ freezeSolPaymentEnabled: boolean;
  /* 18 */ freezeTokenPaymentEnabled: boolean;
};

const GUARDS_SIZE = {
  /* 01 */ botTax: 9,
  /* 02 */ solPayment: 40,
  /* 03 */ tokenPayment: 72,
  /* 04 */ startDate: 8,
  /* 05 */ thirdPartySigner: 32,
  /* 06 */ tokenGate: 40,
  /* 07 */ gatekeeper: 33,
  /* 08 */ endDate: 8,
  /* 09 */ allowList: 32,
  /* 10 */ mintLimit: 3,
  /* 11 */ nftPayment: 64,
  /* 12 */ redeemedAmount: 8,
  /* 13 */ addressGate: 32,
  /* 14 */ nftGate: 32,
  /* 15 */ nftBurn: 32,
  /* 16 */ tokenBurn: 40,
  /* 17 */ freezeSolPayment: 40,
  /* 18 */ freezeTokenPayment: 72,
};
const GUARDS_COUNT = 18;
const MAX_LABEL_LENGTH = 6;

function determineGuards(buffer: Buffer): Guards {
  const enabled = new BN(beet.u64.read(buffer, 0)).toNumber();

  const guards: boolean[] = [];
  for (let i = 0; i < GUARDS_COUNT; i++) {
    guards.push(!!((1 << i) & enabled));
  }

  const [
    botTaxEnabled,
    solPaymentEnabled,
    tokenPaymentEnabled,
    startDateEnabled,
    thirdPartySignerEnabled,
    tokenGateEnabled,
    gatekeeperEnabled,
    endDateEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
    redeemedAmountEnabled,
    addressGateEnabled,
    nftGateEnabled,
    nftBurnEnabled,
    tokenBurnEnabled,
    freezeSolPaymentEnabled,
    freezeTokenPaymentEnabled,
  ] = guards;

  return {
    botTaxEnabled,
    solPaymentEnabled,
    tokenPaymentEnabled,
    startDateEnabled,
    thirdPartySignerEnabled,
    tokenGateEnabled,
    gatekeeperEnabled,
    endDateEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
    redeemedAmountEnabled,
    addressGateEnabled,
    nftGateEnabled,
    nftBurnEnabled,
    tokenBurnEnabled,
    freezeSolPaymentEnabled,
    freezeTokenPaymentEnabled,
  };
}

export function parseData(buffer: Buffer): CandyGuardData {
  // parses the default guard set
  const { guardSet: defaultSet, offset } = parseGuardSet(buffer);
  // retrieves the number of groups
  const groupsCount = new BN(beet.u32.read(buffer, offset)).toNumber();
  const groups: Group[] = [];

  let cursor = beet.u32.byteSize + offset;
  for (let i = 0; i < groupsCount; i++) {
    // parses each individual group
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

function parseGuardSet(buffer: Buffer): { guardSet: GuardSet; offset: number } {
  const guards = determineGuards(buffer);
  const {
    botTaxEnabled,
    startDateEnabled,
    solPaymentEnabled,
    tokenPaymentEnabled,
    thirdPartySignerEnabled,
    tokenGateEnabled,
    gatekeeperEnabled,
    endDateEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
    redeemedAmountEnabled,
    addressGateEnabled,
    nftGateEnabled,
    nftBurnEnabled,
    tokenBurnEnabled,
    freezeSolPaymentEnabled,
    freezeTokenPaymentEnabled,
  } = guards;
  logDebug('Guards: %O', guards);

  // data offset for deserialization (skip u64 features flag)
  let cursor = beet.u64.byteSize;
  // deserialized guards
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  const data: Record<string, any> = {};

  if (botTaxEnabled) {
    const [botTax] = botTaxBeet.deserialize(buffer, cursor);
    data.botTax = botTax;
    cursor += GUARDS_SIZE.botTax;
  }

  if (solPaymentEnabled) {
    const [solPayment] = solPaymentBeet.deserialize(buffer, cursor);
    data.solPayment = solPayment;
    cursor += GUARDS_SIZE.solPayment;
  }

  if (tokenPaymentEnabled) {
    const [tokenPayment] = tokenPaymentBeet.deserialize(buffer, cursor);
    data.tokenPayment = tokenPayment;
    cursor += GUARDS_SIZE.tokenPayment;
  }

  if (startDateEnabled) {
    const [startDate] = startDateBeet.deserialize(buffer, cursor);
    data.startDate = startDate;
    cursor += GUARDS_SIZE.startDate;
  }

  if (thirdPartySignerEnabled) {
    const [thirdPartySigner] = thirdPartySignerBeet.deserialize(buffer, cursor);
    data.thirdPartySigner = thirdPartySigner;
    cursor += GUARDS_SIZE.thirdPartySigner;
  }

  if (tokenGateEnabled) {
    const [tokenGate] = tokenGateBeet.deserialize(buffer, cursor);
    data.tokenGate = tokenGate;
    cursor += GUARDS_SIZE.tokenGate;
  }

  if (gatekeeperEnabled) {
    const [gatekeeper] = gatekeeperBeet.deserialize(buffer, cursor);
    data.gatekeeper = gatekeeper;
    cursor += GUARDS_SIZE.gatekeeper;
  }

  if (endDateEnabled) {
    const [endDate] = endDateBeet.deserialize(buffer, cursor);
    data.endDate = endDate;
    cursor += GUARDS_SIZE.endDate;
  }

  if (allowListEnabled) {
    const [allowList] = allowListBeet.deserialize(buffer, cursor);
    data.allowList = allowList;
    cursor += GUARDS_SIZE.allowList;
  }

  if (mintLimitEnabled) {
    const [mintLimit] = mintLimitBeet.deserialize(buffer, cursor);
    data.mintLimit = mintLimit;
    cursor += GUARDS_SIZE.mintLimit;
  }

  if (nftPaymentEnabled) {
    const [nftPayment] = nftPaymentBeet.deserialize(buffer, cursor);
    data.nftPayment = nftPayment;
    cursor += GUARDS_SIZE.nftPayment;
  }

  if (redeemedAmountEnabled) {
    const [redeemedAmount] = redeemedAmountBeet.deserialize(buffer, cursor);
    data.redeemedAmount = redeemedAmount;
    cursor += GUARDS_SIZE.redeemedAmount;
  }

  if (addressGateEnabled) {
    const [addressGate] = addressGateBeet.deserialize(buffer, cursor);
    data.addressGate = addressGate;
    cursor += GUARDS_SIZE.addressGate;
  }

  if (nftGateEnabled) {
    const [nftGate] = nftGateBeet.deserialize(buffer, cursor);
    data.nftGate = nftGate;
    cursor += GUARDS_SIZE.nftGate;
  }

  if (nftBurnEnabled) {
    const [nftBurn] = nftBurnBeet.deserialize(buffer, cursor);
    data.nftBurn = nftBurn;
    cursor += GUARDS_SIZE.nftBurn;
  }

  if (tokenBurnEnabled) {
    const [tokenBurn] = tokenBurnBeet.deserialize(buffer, cursor);
    data.tokenBurn = tokenBurn;
    cursor += GUARDS_SIZE.tokenBurn;
  }

  if (freezeSolPaymentEnabled) {
    const [freezeSolPayment] = freezeSolPaymentBeet.deserialize(buffer, cursor);
    data.freezeSolPayment = freezeSolPayment;
    cursor += GUARDS_SIZE.freezeSolPayment;
  }

  if (freezeTokenPaymentEnabled) {
    const [freezeTokenPayment] = freezeTokenPaymentBeet.deserialize(buffer, cursor);
    data.freezeTokenPayment = freezeTokenPayment;
    cursor += GUARDS_SIZE.freezeTokenPayment;
  }

  return {
    guardSet: {
      botTax: data.botTax ?? null,
      solPayment: data.solPayment ?? null,
      tokenPayment: data.tokenPayment ?? null,
      startDate: data.startDate ?? null,
      thirdPartySigner: data.thirdPartySigner ?? null,
      tokenGate: data.tokenGate ?? null,
      gatekeeper: data.gateKeeper ?? null,
      endDate: data.endDate ?? null,
      allowList: data.allowList ?? null,
      mintLimit: data.mintLimit ?? null,
      nftPayment: data.nftPayment ?? null,
      redeemedAmount: data.redeemedAmount ?? null,
      addressGate: data.addressGate ?? null,
      nftGate: data.nftGate ?? null,
      nftBurn: data.nftBurn ?? null,
      tokenBurn: data.tokenBurn ?? null,
      freezeSolPayment: data.freezeSolPayment ?? null,
      freezeTokenPayment: data.freezeTokenPayment ?? null,
    },
    offset: cursor,
  };
}
