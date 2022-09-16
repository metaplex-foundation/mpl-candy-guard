import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../src/generated';

const API = new InitTransactions();

killStuckProcess();

test('mint limit', async (t) => {
  // deploys a candy guard with a mint limit

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
      mintLimit: {
        id: 0,
        limit: 1,
      },
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
    minterPair: minterKeypair,
    connection: minterConnection,
  } = await API.minter();

  const [mintCounterPda] = await PublicKey.findProgramAddress(
    [
      new Uint8Array([0]),
      minterKeypair.publicKey.toBuffer(),
      candyGuard.toBuffer(),
      candyMachine.toBuffer(),
    ],
    PROGRAM_ID,
  );

  // limit is 1, this should succeed

  const [, mintForMinter] = await amman.genLabeledKeypair('Mint Account 1 (minter)');
  const { tx: minterMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinter,
    minterHandler,
    minterConnection,
    [
      {
        pubkey: mintCounterPda,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx.assertSuccess(t);

  // limit is 1, this should fail

  const [, mintForMinter2] = await amman.genLabeledKeypair('Mint Account 2 (minter)');
  const { tx: minterMintTx2 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinter2,
    minterHandler,
    minterConnection,
    [
      {
        pubkey: mintCounterPda,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx2.assertError(t, /maximum number of allowed mints/i);

  // another minter can mint

  const {
    fstTxHandler: minter2Handler,
    authorityPair: minter2Keypair,
    connection: minter2Connection,
  } = await API.authority();

  const [mintCounterPda2] = await PublicKey.findProgramAddress(
    [
      new Uint8Array([0]),
      minter2Keypair.publicKey.toBuffer(),
      candyGuard.toBuffer(),
      candyMachine.toBuffer(),
    ],
    PROGRAM_ID,
  );

  const [, mintForMinter3] = await amman.genLabeledKeypair('Mint Account (minter 2)');
  const { tx: minterMintTx3 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter2Keypair,
    mintForMinter3,
    minter2Handler,
    minter2Connection,
    [
      {
        pubkey: mintCounterPda2,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx3.assertSuccess(t);
});

test('mint limit (limit = 2)', async (t) => {
  // deploys a candy guard with a mint limit

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
      mintLimit: {
        id: 0,
        limit: 2,
      },
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
    minterPair: minterKeypair,
    connection: minterConnection,
  } = await API.minter();

  const [mintCounterPda] = await PublicKey.findProgramAddress(
    [
      new Uint8Array([0]),
      minterKeypair.publicKey.toBuffer(),
      candyGuard.toBuffer(),
      candyMachine.toBuffer(),
    ],
    PROGRAM_ID,
  );

  // limit is 2, this should succeed

  const [, mintForMinter] = await amman.genLabeledKeypair('Mint Account 1 (minter)');
  const { tx: minterMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinter,
    minterHandler,
    minterConnection,
    [
      {
        pubkey: mintCounterPda,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx.assertSuccess(t);

  // limit is 2, this should succeed

  const [, mintForMinte2] = await amman.genLabeledKeypair('Mint Account 2 (minter)');
  const { tx: minterMintTx2 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinte2,
    minterHandler,
    minterConnection,
    [
      {
        pubkey: mintCounterPda,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx2.assertSuccess(t);

  // limit is 2, this should fail

  const [, mintForMinter3] = await amman.genLabeledKeypair('Mint Account 3 (minter)');
  const { tx: minterMintTx3 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinter3,
    minterHandler,
    minterConnection,
    [
      {
        pubkey: mintCounterPda,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx3.assertError(t, /maximum number of allowed mints/i);

  // another minter can mint

  const {
    fstTxHandler: minter2Handler,
    authorityPair: minter2Keypair,
    connection: minter2Connection,
  } = await API.authority();

  const [mintCounterPda2] = await PublicKey.findProgramAddress(
    [
      new Uint8Array([0]),
      minter2Keypair.publicKey.toBuffer(),
      candyGuard.toBuffer(),
      candyMachine.toBuffer(),
    ],
    PROGRAM_ID,
  );

  const [, mintForMinter4] = await amman.genLabeledKeypair('Mint Account (minter 2)');
  const { tx: minterMintTx4 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter2Keypair,
    mintForMinter4,
    minter2Handler,
    minter2Connection,
    [
      {
        pubkey: mintCounterPda2,
        isSigner: false,
        isWritable: true,
      },
    ],
  );

  await minterMintTx4.assertSuccess(t);
});
