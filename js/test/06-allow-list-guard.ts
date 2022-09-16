import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { MerkleTree } from 'merkletreejs';
import { keccak_256 } from '@noble/hashes/sha3';
import { u32 } from '@metaplex-foundation/beet';

const API = new InitTransactions();

killStuckProcess();

test('allowlist', async (t) => {
  const addresses: string[] = [];

  // list of addresses in the allow list

  for (let i = 0; i < 9; i++) {
    const [address] = await amman.genLabeledKeypair(`Wallet ${i}`);
    addresses.push(address.toString());
  }

  const {
    fstTxHandler: minterHandler,
    minterPair: minterKeypair,
    connection: minterConnection,
  } = await API.minter();
  addresses.push(minterKeypair.publicKey.toString());

  // creates the merkle tree
  const tree = new MerkleTree(addresses.map(keccak_256), keccak_256, { sortPairs: true });

  // deploys a candy guard with the allow list – the allowList guard is configured
  // with the root of the merkle tree

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
      allowList: {
        merkleRoot: [...tree.getRoot()],
      },
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

  // the proof will be empty if the address is not found in the merkle tree
  const proof = tree.getProof(Buffer.from(keccak_256(minterKeypair.publicKey.toString())));

  const vectorSizeBuffer = Buffer.alloc(4);
  u32.write(vectorSizeBuffer, 0, proof.length);

  const leafBuffers = proof.map((leaf) => leaf.data);
  // prepares the mint arguments with the merkle proof
  const mintArgs = Buffer.concat([vectorSizeBuffer, ...leafBuffers]);

  const [, mintForMinter] = await amman.genLabeledKeypair('Mint Account (minter)');
  const { tx: minterMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    minterKeypair,
    mintForMinter,
    minterHandler,
    minterConnection,
    null,
    mintArgs,
  );

  await minterMintTx.assertSuccess(t);

  // trying to mint (as a authority) reusing the proof of another wallet

  const [, mintForPayer] = await amman.genLabeledKeypair('Mint Account (authority)');
  const { tx: payerMintTx } = await API.mint(
    t,
    candyGuard,
    candyMachine,
    payerPair,
    mintForPayer,
    fstTxHandler,
    connection,
    null,
    mintArgs,
  );

  await payerMintTx.assertError(t, /Address not found/i);
});
