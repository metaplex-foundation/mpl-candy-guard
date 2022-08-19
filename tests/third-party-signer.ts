import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";
import { CandyMachine } from "../target/types/candy_machine";
import * as test from "./helpers";

describe("Third Party Signer", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  // candy guard for the tests
  const candyGuardBaseKeypair = anchor.web3.Keypair.generate();
  // candy machine for the tests
  const candyMachineKeypair = anchor.web3.Keypair.generate();
  // third party signer
  const signer = anchor.web3.Keypair.generate();
  // candy guard program
  const candyGuardProgram = anchor.workspace.CandyGuard as Program<CandyGuard>;
  // candy machine program
  const candyMachineProgram = anchor.workspace.CandyMachine as Program<CandyMachine>;
  // payer of the transactions
  const payer = (candyGuardProgram.provider as anchor.AnchorProvider).wallet as Wallet;

  /**
   * Initializes a new candy guard.
   */
  it("initialize", async () => {
    let candyGuardKey = await test.createCandyGuard(candyGuardProgram, candyGuardBaseKeypair, payer);
    let candyGuard = await candyGuardProgram.account.candyGuard.fetch(candyGuardKey);
    expect(candyGuard.features.toNumber()).to.equal(0);
  });

  /**
   * Updates the candy guard configuration.
   */
  it("update", async () => {
    let candyGuardKey = await test.getCandyGuardPDA(candyGuardProgram, candyGuardBaseKeypair);
    let candyGuard = await candyGuardProgram.account.candyGuard.fetch(candyGuardKey);
    // no features enabled by default
    expect(candyGuard.features.toNumber()).to.equal(0);

    // enabling third party signer

    let settings = test.defaultCandyGuardSettings();
    settings.botTax = null;
    settings.liveDate = null;
    settings.lamportsCharge = null;
    settings.spltokenCharge = null;
    settings.thirdPartySigner.signerKey = signer.publicKey;
    settings.whitelist = null;

    await candyGuardProgram.methods.update(settings).accounts({
      candyGuard: candyGuardKey,
      authority: payer.publicKey,
    }).rpc();

    candyGuard = await candyGuardProgram.account.candyGuard.fetch(candyGuardKey);
    expect(candyGuard.features.toNumber()).to.equal(0b10000);
  });

  /**
   * Initializes a candy machine.
   */
  it("create candy machine", async () => {
    const items = 10;
    const data = test.defaultCandyMachineSettings(items, payer.publicKey, false);
    await test.createCandyMachine(candyMachineProgram, candyMachineKeypair, payer, data);

    let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);
    const lines = [];

    for (let i = 0; i < candyMachine.data.itemsAvailable.toNumber(); i++) {
      const line = JSON.parse(`{\
                "name": "NFT #${i + 1}",\
                "uri": "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"\
            }`);

      lines[i] = line;
    }

    await candyMachineProgram.methods.addConfigLines(0, lines).accounts({
      candyMachine: candyMachineKeypair.publicKey,
      authority: payer.publicKey,
    }).rpc();
  });

  /**
   * Mint an item from the candy machine.
   */
  it("wrap", async () => {
    // candy guard PDA
    const [pda,] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('candy_guard'), candyGuardBaseKeypair.publicKey.toBuffer()],
      candyGuardProgram.programId,
    );

    await candyGuardProgram.methods.wrap().accounts({
      candyGuard: pda,
      candyMachine: candyMachineKeypair.publicKey,
      candyMachineProgram: candyMachineProgram.programId,
      authority: payer.publicKey,
    }).rpc();
  });

  /**
   * Mints from the candy guard.
   */
  it("mint", async () => {
    let failed = false;
    try {
      // this should fail since it is missing the third-party signer
      await test.mintFromCandyGuard(
        candyGuardProgram,
        candyMachineProgram,
        candyGuardBaseKeypair,
        candyMachineKeypair,
        payer
      );
    } catch {
      failed = true;
    }
    expect(failed).equal(true);

    // try again with the signer
    const signature = await test.mintFromCandyGuard(
      candyGuardProgram,
      candyMachineProgram,
      candyGuardBaseKeypair,
      candyMachineKeypair,
      payer,
      signer
    );
    console.log(signature);

  });

  /**
   * Withdraw the rent from the candy machine.
   */
  it("withdraw", async () => {
    // candy guard PDA
    const [pda,] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('candy_guard'), candyGuardBaseKeypair.publicKey.toBuffer()],
      candyGuardProgram.programId,
    );

    await candyGuardProgram.methods.unwrap().accounts({
      candyGuard: pda,
      candyMachine: candyMachineKeypair.publicKey,
      candyMachineProgram: candyMachineProgram.programId,
      authority: payer.publicKey,
    }).rpc();

    await candyMachineProgram.methods.withdraw().accounts({
      candyMachine: candyMachineKeypair.publicKey,
      authority: payer.publicKey,
    }).rpc();
  });
});
