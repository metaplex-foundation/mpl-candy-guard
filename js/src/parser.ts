import { BN } from 'bn.js';
import * as beet from '@metaplex-foundation/beet';
import { logDebug } from './utils/log';
import * as beets from './generated';
import { BeetArgsStruct, u32, u64 } from '@metaplex-foundation/beet';
import { CandyGuardData, Group, GuardSet } from './generated';
import { strict as assert } from 'assert';

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
  /* 19 */ programGateEnabled: boolean;
};

const GUARDS_NAME = [
  /* 01 */ 'botTax',
  /* 02 */ 'solPayment',
  /* 03 */ 'tokenPayment',
  /* 04 */ 'startDate',
  /* 05 */ 'thirdPartySigner',
  /* 06 */ 'tokenGate',
  /* 07 */ 'gatekeeper',
  /* 08 */ 'endDate',
  /* 09 */ 'allowList',
  /* 10 */ 'mintLimit',
  /* 11 */ 'nftPayment',
  /* 12 */ 'redeemedAmount',
  /* 13 */ 'addressGate',
  /* 14 */ 'nftGate',
  /* 15 */ 'nftBurn',
  /* 16 */ 'tokenBurn',
  /* 17 */ 'freezeSolPayment',
  /* 18 */ 'freezeTokenPayment',
  /* 19 */ 'programGate',
] as const;

type GuardsKey = typeof GUARDS_NAME[number];

const GUARDS_SIZE: Record<GuardsKey, number> = {
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
  /* 19 */ programGate: 164,
};

const GUARDS_COUNT = GUARDS_NAME.length;
const MAX_LABEL_LENGTH = 6;
const MAX_PROGRAM_COUNT = 5;

/**
 * Returns the guards that are enabled.
 *
 * @param buffer bytes representing the guards data.
 *
 * @returns a `Guards` object.
 */
function guardsFromData(buffer: Buffer): Guards {
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
    programGateEnabled,
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
    programGateEnabled,
  };
}

/**
 * Returns a `CandyGuardData` object from a data buffer.
 *
 * @param buffer bytes representing the Candy Guard data.
 *
 * @returns a `CandyGuardData` object from a data buffer.
 */
export function deserialize(buffer: Buffer): CandyGuardData {
  // parses the default guard set
  const { guardSet: defaultSet, offset } = deserializeGuardSet(buffer);
  // retrieves the number of groups
  const groupsCount = new BN(beet.u32.read(buffer, offset)).toNumber();
  const groups: Group[] = [];

  let cursor = beet.u32.byteSize + offset;
  for (let i = 0; i < groupsCount; i++) {
    // parses each individual group
    const label = buffer.subarray(cursor, cursor + MAX_LABEL_LENGTH).toString();
    cursor += MAX_LABEL_LENGTH;
    const { guardSet: guards, offset } = deserializeGuardSet(buffer.subarray(cursor));
    groups.push({ label, guards });
    cursor += offset;
  }

  return {
    default: defaultSet,
    groups: groups.length === 0 ? null : groups,
  };
}

/**
 * Serializes the Cnady Guard data to a byte buffer.
 *
 * @param data the Candy Guard data to be serialized.
 *
 * @returns byte buffer.
 */
export function serialize(data: CandyGuardData): Buffer {
  const buffer = Buffer.alloc(size(data));
  // serializes the default guard set
  let offset = serializeGuardSet(buffer, 0, data.default);

  // write the number of groups
  const groupsCount = data.groups ? data.groups.length : 0;
  u32.write(buffer, offset, groupsCount);
  offset += u32.byteSize;

  for (let i = 0; i < groupsCount; i++) {
    // serializes each individual group
    const group = data.groups!.at(i);
    // label
    if (group!.label.length > MAX_LABEL_LENGTH) {
      throw `Exceeded maximum label length: ${group!.label.length} > ${MAX_LABEL_LENGTH}`;
    }
    buffer.write(group!.label, offset, MAX_LABEL_LENGTH, 'utf8');
    offset += MAX_LABEL_LENGTH;
    // guards
    offset = serializeGuardSet(buffer, offset, group!.guards);
  }

  return buffer;
}

/**
 * Returns the number of bytes needed to serialize the specified
 * `CandyGuardData` object.
 *
 * @param data the `CandyGuardData` object.
 *
 * @returns the number of bytes needed to serialize the specified
 * `CandyGuardData` object.
 */
function size(data: CandyGuardData): number {
  let size = guardSetSize(data.default);
  size += u32.byteSize;

  if (data.groups) {
    for (let i = 0; i < data.groups.length; i++) {
      size += MAX_LABEL_LENGTH;
      size += guardSetSize(data.groups.at(i)!.guards);
    }
  }

  return size;
}

/**
 * Returns the number of bytes needed to serialize the specified
 * `GuardSet` object.
 *
 * @param guardSet the `GuardSet` object.
 *
 * @returns the number of bytes needed to serialize the specified
 * `GuardSet` object.
 */
function guardSetSize(guardSet: GuardSet): number {
  type ObjectKey = keyof typeof guardSet;
  const guards: number[] = [];
  for (let i = 0; i < GUARDS_COUNT; i++) {
    const index = GUARDS_NAME[i] as ObjectKey;
    if (guardSet[index]) {
      guards.push(GUARDS_SIZE[index]);
    }
  }

  // features flag + guards data
  return (
    u64.byteSize + guards.reduce((previousValue, currentValue) => previousValue + currentValue, 0)
  );
}

function addGuardIfEnabled(
  guards: Guards,
  guardsSize: typeof GUARDS_SIZE,
  key: GuardsKey,
  buffer: Buffer,
  cursor: number,
  data: Partial<Record<GuardsKey, any>>,
): number {
  const enbledKey: keyof Guards = `${key}Enabled`;
  if (!guards[enbledKey]) return cursor;

  const beetKey = `${key}Beet` as keyof typeof beet;
  const beet = (beets as any)[beetKey] as BeetArgsStruct<any>;
  assert(beet != null, `Beet for ${key} not found`);
  assert(typeof beet.deserialize === 'function', `Beet for ${key} is missing deserialize function`);

  const [val] = beet.deserialize(buffer, cursor);
  data[key] = val;
  return cursor + guardsSize[key];
}

/**
 * Returns a `GuardSet` object from the byte buffer.
 *
 * @param buffer the byte buffer to read from.
 *
 * @returns an object with a `GuardSet` object from the byte buffer and
 * the number of bytes (offset) consumed.
 */
function deserializeGuardSet(buffer: Buffer): { guardSet: GuardSet; offset: number } {
  const guards = guardsFromData(buffer);
  logDebug('Guards: %O', guards);

  // data offset for deserialization (skip u64 features flag)
  let cursor = beet.u64.byteSize;
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  const data: Partial<Record<GuardsKey, any>> = {};

  for (const key of GUARDS_NAME) {
    cursor = addGuardIfEnabled(guards, GUARDS_SIZE, key, buffer, cursor, data);
  }

  return {
    guardSet: {
      botTax: data.botTax ?? null,
      solPayment: data.solPayment ?? null,
      tokenPayment: data.tokenPayment ?? null,
      startDate: data.startDate ?? null,
      thirdPartySigner: data.thirdPartySigner ?? null,
      tokenGate: data.tokenGate ?? null,
      gatekeeper: data.gatekeeper ?? null,
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
      programGate: data.programGate ?? null,
    },
    offset: cursor,
  };
}

/**
 * Serializes a `GuardSet` object to the specified buffer.
 *
 * @param buffer the byte buffer to write to.
 * @param offset the byte offset.
 * @param guardSet the `GuardSet` object.
 *
 * @returns the byte offset at the end of the serialization.
 */
function serializeGuardSet(buffer: Buffer, offset: number, guardSet: GuardSet): number {
  // saves the initial position to write the features flag
  const start = offset;
  // skip the bytes for the feature flag
  offset += u64.byteSize;

  let features = 0;
  let index = 0;

  if (guardSet.botTax) {
    beets.botTaxBeet.write(buffer, offset, guardSet.botTax);
    offset += GUARDS_SIZE.botTax;
    features |= 1 << index;
  }
  index++;

  if (guardSet.solPayment) {
    beets.solPaymentBeet.write(buffer, offset, guardSet.solPayment);
    offset += GUARDS_SIZE.solPayment;
    features |= 1 << index;
  }
  index++;

  if (guardSet.tokenPayment) {
    beets.tokenPaymentBeet.write(buffer, offset, guardSet.tokenPayment);
    offset += GUARDS_SIZE.tokenPayment;
    features |= 1 << index;
  }
  index++;

  if (guardSet.startDate) {
    beets.startDateBeet.write(buffer, offset, guardSet.startDate);
    offset += GUARDS_SIZE.startDate;
    features |= 1 << index;
  }
  index++;

  if (guardSet.thirdPartySigner) {
    beets.thirdPartySignerBeet.write(buffer, offset, guardSet.thirdPartySigner);
    offset += GUARDS_SIZE.thirdPartySigner;
    features |= 1 << index;
  }
  index++;

  if (guardSet.tokenGate) {
    beets.tokenGateBeet.write(buffer, offset, guardSet.tokenGate);
    offset += GUARDS_SIZE.tokenGate;
    features |= 1 << index;
  }
  index++;

  if (guardSet.gatekeeper) {
    beets.gatekeeperBeet.write(buffer, offset, guardSet.gatekeeper);
    offset += GUARDS_SIZE.gatekeeper;
    features |= 1 << index;
  }
  index++;

  if (guardSet.endDate) {
    beets.endDateBeet.write(buffer, offset, guardSet.endDate);
    offset += GUARDS_SIZE.endDate;
    features |= 1 << index;
  }
  index++;

  if (guardSet.allowList) {
    beets.allowListBeet.write(buffer, offset, guardSet.allowList);
    offset += GUARDS_SIZE.allowList;
    features |= 1 << index;
  }
  index++;

  if (guardSet.mintLimit) {
    beets.mintLimitBeet.write(buffer, offset, guardSet.mintLimit);
    offset += GUARDS_SIZE.mintLimit;
    features |= 1 << index;
  }
  index++;

  if (guardSet.nftPayment) {
    beets.nftPaymentBeet.write(buffer, offset, guardSet.nftPayment);
    offset += GUARDS_SIZE.nftPayment;
    features |= 1 << index;
  }
  index++;

  if (guardSet.redeemedAmount) {
    beets.redeemedAmountBeet.write(buffer, offset, guardSet.redeemedAmount);
    offset += GUARDS_SIZE.redeemedAmount;
    features |= 1 << index;
  }
  index++;

  if (guardSet.addressGate) {
    beets.addressGateBeet.write(buffer, offset, guardSet.addressGate);
    offset += GUARDS_SIZE.addressGate;
    features |= 1 << index;
  }
  index++;

  if (guardSet.nftGate) {
    beets.nftGateBeet.write(buffer, offset, guardSet.nftGate);
    offset += GUARDS_SIZE.nftGate;
    features |= 1 << index;
  }
  index++;

  if (guardSet.nftBurn) {
    beets.nftBurnBeet.write(buffer, offset, guardSet.nftBurn);
    offset += GUARDS_SIZE.nftBurn;
    features |= 1 << index;
  }
  index++;

  if (guardSet.tokenBurn) {
    beets.tokenBurnBeet.write(buffer, offset, guardSet.tokenBurn);
    offset += GUARDS_SIZE.tokenBurn;
    features |= 1 << index;
  }
  index++;

  if (guardSet.freezeSolPayment) {
    beets.freezeSolPaymentBeet.write(buffer, offset, guardSet.freezeSolPayment);
    offset += GUARDS_SIZE.freezeSolPayment;
    features |= 1 << index;
  }
  index++;

  if (guardSet.freezeTokenPayment) {
    beets.freezeTokenPaymentBeet.write(buffer, offset, guardSet.freezeTokenPayment);
    offset += GUARDS_SIZE.freezeTokenPayment;
    features |= 1 << index;
  }
  index++;

  if (guardSet.programGate) {
    if (
      guardSet.programGate.additional &&
      guardSet.programGate.additional.length > MAX_PROGRAM_COUNT
    ) {
      throw `Exceeded maximum number of programs on additional list:\
        ${guardSet.programGate.additional.length} > ${MAX_PROGRAM_COUNT}`;
    }

    const [data] = beets.programGateBeet.serialize(guardSet.programGate, GUARDS_SIZE.programGate);
    data.copy(buffer, offset);
    offset += GUARDS_SIZE.programGate;
    features |= 1 << index;
  }
  index++;

  u64.write(buffer, start, features);

  return offset;
}
