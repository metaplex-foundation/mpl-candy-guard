import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';

const API = new InitTransactions();

killStuckProcess();

test('end date (in the past)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      startDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      tokenGate: null,
      gatekeeper: null,
      endDate: {
        date: 1663979606,
      },
      allowList: null,
      mintLimit: null,
      nftPayment: null,
      redemeedAmount: null,
      addressGate: null,
    },
    groups: null,
  };

  const { candyGuard, candyMachine } = await API.deploy(
    t,
    data,
    payerPair,
    fstTxHandler,
    connection,
  );

  // mint

  const [, mintForAuthority] = await amman.genLabeledKeypair('Mint Account (authority)');
  const { tx: authorityMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    payerPair,
    mintForAuthority,
    fstTxHandler,
    connection,
  );
  await authorityMintTx.assertError(t, /time is after/i);
});

test('end date (in the future)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      startDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      tokenGate: null,
      gatekeeper: null,
      endDate: {
        date: 32534611200,
      },
      allowList: null,
      mintLimit: null,
      nftPayment: null,
      redemeedAmount: null,
      addressGate: null,
    },
    groups: null,
  };

  const { candyGuard, candyMachine } = await API.deploy(
    t,
    data,
    payerPair,
    fstTxHandler,
    connection,
  );

  // mint

  const {
    fstTxHandler: minterHandler,
    minterPair: minter,
    connection: minterConnection,
  } = await API.minter();
  const [, mintForMinter] = await amman.genLabeledKeypair('Mint Account (minter)');
  const { tx: minterMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter,
    mintForMinter,
    minterHandler,
    minterConnection,
  );
  await minterMintTx.assertSuccess(t);
});
