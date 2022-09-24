import test from 'tape';
import spok from 'spok';
import { BN } from 'bn.js';
import { newCandyGuardData, InitTransactions, killStuckProcess } from './setup/';
import { CandyGuard } from '../src/generated';
import { DATA_OFFSET, spokSameBignum, spokSamePubkey } from './utils';
import { parseData } from '../src';

const API = new InitTransactions();

killStuckProcess();

test('initialize: new candy guard (no guards)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = newCandyGuardData();

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  // parse the guards configuration
  const accountInfo = await connection.getAccountInfo(address);
  const candyGuardData = parseData(accountInfo!.data.subarray(DATA_OFFSET));

  spok(t, candyGuardData, data);
});

test('initialize: new candy guard (with guards)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = newCandyGuardData();
  data.default.botTax = {
    lamports: new BN(100000000),
    lastInstruction: true,
  };
  data.default.solPayment = {
    lamports: new BN(100000000),
    destination: payerPair.publicKey,
  };
  data.default.startDate = {
    date: 1663965742,
  };
  data.default.thirdPartySigner = {
    signerKey: payerPair.publicKey,
  };

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  // parse the guards configuration
  const accountInfo = await connection.getAccountInfo(address);
  const candyGuardData = parseData(accountInfo!.data.subarray(DATA_OFFSET)!);

  spok(t, candyGuardData.default.botTax, {
    lamports: spokSameBignum(data.default.botTax.lamports),
    lastInstruction: true,
  });

  spok(t, candyGuardData.default.startDate, {
    date: spokSameBignum(data.default.startDate.date),
  });

  spok(t, candyGuardData.default.solPayment, {
    lamports: spokSameBignum(data.default.solPayment.lamports),
  });

  spok(t, candyGuardData.default.thirdPartySigner, {
    signerKey: spokSamePubkey(payerPair.publicKey),
  });
});
