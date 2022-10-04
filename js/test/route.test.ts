import test from 'tape';
import spok from 'spok';
import { newCandyGuardData, newGuardSet, InitTransactions, killStuckProcess } from './setup';
import { CandyGuard, createRouteInstruction, RouteInstructionAccounts, RouteInstructionArgs } from '../src/generated';
import { DATA_OFFSET, spokSameBignum, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { parseData } from '../src';
import { Transaction } from '@solana/web3.js';

killStuckProcess();

test('route', async (t) => {
  const API = new InitTransactions();
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

  //let accountInfo = await connection.getAccountInfo(payerPair.publicKey);

  const accounts: RouteInstructionAccounts = {
    candyGuard: address,
  };

  const args: RouteInstructionArgs = {
    data: new Uint8Array(),
  };

  const tx = new Transaction().add(createRouteInstruction(accounts, args));
  const h = fstTxHandler.sendAndConfirmTransaction(tx, [payerPair], 'tx: Route');

  await h.assertSuccess(t);
});
