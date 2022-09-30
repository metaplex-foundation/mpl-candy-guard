import test from 'tape';
import { amman, InitTransactions, killStuckProcess, newCandyGuardData } from '../setup';
import { Metaplex, keypairIdentity } from '@metaplex-foundation/js';
import { AccountMeta, PublicKey } from '@solana/web3.js';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { CandyMachine } from '@metaplex-foundation/mpl-candy-machine-core';

const API = new InitTransactions();

killStuckProcess();

test('nft payment', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = newCandyGuardData();
  data.default.startDate = {
    date: 1662479807,
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
  await minterMintTx.assertSuccess(t);

  // enables the nft_payment guard

  const [tokenAccount] = await PublicKey.findProgramAddress(
    [minter.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mintForMinter.publicKey.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);

  const updatedData = newCandyGuardData();
  updatedData.default.startDate = {
    date: 1662479807,
  };
  updatedData.default.nftPayment = {
    requiredCollection: candyMachineObject.collectionMint,
    destination: payerPair.publicKey,
  };

  const { tx: updateTx } = await API.update(t, candyGuard, updatedData, payerPair, fstTxHandler);
  await updateTx.assertSuccess(t);

  // mint (as a minter)

  const [, mintForMinter2] = await amman.genLabeledKeypair('Mint Account 2 (minter)');
  const { tx: minterMintTx2 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter,
    mintForMinter2,
    minterHandler,
    minterConnection,
  );
  await minterMintTx2.assertError(t, /Missing expected remaining account/i);

  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));
  const nft = await metaplex.nfts().findByMint({ mintAddress: mintForMinter.publicKey }).run();
  const paymentGuardAccounts: AccountMeta[] = [];

  // nft account
  paymentGuardAccounts.push({
    pubkey: tokenAccount,
    isSigner: false,
    isWritable: true,
  });
  // nft metadata
  paymentGuardAccounts.push({
    pubkey: nft.metadataAddress,
    isSigner: false,
    isWritable: true,
  });
  // nft mint
  paymentGuardAccounts.push({
    pubkey: mintForMinter.publicKey,
    isSigner: false,
    isWritable: false,
  });
  // destination
  paymentGuardAccounts.push({
    pubkey: updatedData.default.nftPayment.destination,
    isSigner: false,
    isWritable: false,
  });
  // destination ATA
  const [destinationAta] = await PublicKey.findProgramAddress(
    [
      updatedData.default.nftPayment.destination.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintForMinter.publicKey.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  paymentGuardAccounts.push({
    pubkey: destinationAta,
    isSigner: false,
    isWritable: true,
  });
  // associate token program
  paymentGuardAccounts.push({
    pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
    isSigner: false,
    isWritable: false,
  });

  const [, mintForMinter3] = await amman.genLabeledKeypair('Mint Account 3 (minter)');
  const { tx: minterMintTx3 } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minter,
    mintForMinter3,
    minterHandler,
    minterConnection,
    paymentGuardAccounts,
  );
  await minterMintTx3.assertSuccess(t);
});
