import test from 'tape';
import spok from 'spok';
import { InitTransactions, killStuckProcess } from './setup';
import { CandyGuard } from '../src/generated';
import { DATA_OFFSET, spokSameBignum, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { parseData } from '../src';
import { CollectionAuthorityDoesNotExistError } from '@metaplex-foundation/mpl-token-metadata';

const API = new InitTransactions();

killStuckProcess();

test('update: enable guards', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      liveDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
    },
    groups: null,
  };

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const balance = accountInfo?.lamports!;

  const updateData = {
    default: {
      botTax: {
        lamports: new BN(100000000),
        lastInstruction: true,
      },
      liveDate: {
        date: null,
      },
      lamports: {
        amount: new BN(100000000),
        destination: address,
      },
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
    },
    groups: null,
  };

  const { tx: updateTransaction } = await API.update(
    t,
    address,
    updateData,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await updateTransaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo?.lamports!;

  t.true(updatedBalance < balance, 'balance after update must be lower');
});

test('update: disable guards', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: {
        lamports: new BN(100000000),
        lastInstruction: true,
      },
      liveDate: {
        date: null,
      },
      lamports: {
        amount: new BN(100000000),
        destination: payerPair.publicKey,
      },
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
    },
    groups: [
      {
        label: 'VIP',
        guards: {
          botTax: null,
          liveDate: {
            date: 1662394820,
          },
          lamports: {
            amount: new BN(500),
            destination: payerPair.publicKey,
          },
          splToken: null,
          thirdPartySigner: null,
          whitelist: null,
          gatekeeper: null,
          endSettings: null,
          allowList: null,
          mintLimit: null,
          nftPayment: null,
        },
      },
      {
        label: 'OGs',
        guards: {
          botTax: null,
          liveDate: null,
          lamports: {
            amount: new BN(1000),
            destination: payerPair.publicKey,
          },
          splToken: null,
          thirdPartySigner: null,
          whitelist: null,
          gatekeeper: null,
          endSettings: null,
          allowList: null,
          mintLimit: null,
          nftPayment: null,
        },
      },
    ],
  };

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  // parse the guards configuration
  let accountInfo = await connection.getAccountInfo(address);
  const candyGuardData = parseData(accountInfo?.data.subarray(DATA_OFFSET)!);

  t.true(candyGuardData.groups?.length === 2, 'expected 2 group2');

  const group1 = candyGuardData.groups?.at(0)!;
  // group 1
  spok(t, group1.label, 'VIP');
  spok(t, group1.guards.liveDate?.date, spokSameBignum(1662394820));
  spok(t, group1.guards.lamports?.amount, spokSameBignum(500));

  const group2 = candyGuardData.groups?.at(1)!;
  // group 2
  spok(t, group2.label, 'OGs');
  spok(t, group2.guards.lamports?.amount, spokSameBignum(1000));

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const balance = accountInfo?.lamports!;

  const updateData = {
    default: {
      botTax: null,
      liveDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
    },
    groups: null,
  };

  const { tx: updateTransaction } = await API.update(
    t,
    address,
    updateData,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await updateTransaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo?.lamports!;

  t.true(updatedBalance > balance, 'balance after update must be greater');
});
