import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';

const API = new InitTransactions();

killStuckProcess();

test('start date (in the past)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      startDate: {
        date: 1662479807,
      },
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      tokenGate: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
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

  // mint (as an authority)

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
  await authorityMintTx.assertSuccess(t);

  // mint (as a minter)

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

test('start date (in the future)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      startDate: {
        date: 1671926400,
      },
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      tokenGate: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
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

  // mint (as a minter)

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
  await minterMintTx.assertError(t, /Mint is not live/i);
});
