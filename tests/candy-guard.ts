import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";
import { createCandyGuard, defaultCandyGuardSettings, getCandyGuardPDA } from "./helpers";

describe("Candy Guard", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  // candy guard for the tests
  const keypair = anchor.web3.Keypair.generate();
  // candy guard program
  const program = anchor.workspace.CandyGuard as Program<CandyGuard>;
  // payer of the transactions
  const payer = (program.provider as anchor.AnchorProvider).wallet as Wallet;

  /**
   * Initializes a new candy guard.
   */
  it("initialize", async () => {
    let candyGuardKey = await createCandyGuard(program, keypair, payer);
    let candyGuard = await program.account.candyGuard.fetch(candyGuardKey);
    expect(candyGuard.features.toNumber()).to.equal(0);
  });

  /**
   * Updates the candy guard configuration.
   */
  it("update", async () => {
    let candyGuardKey = await getCandyGuardPDA(program, keypair);
    let candyGuard = await program.account.candyGuard.fetch(candyGuardKey);
    // no features enabled by default
    expect(candyGuard.features.toNumber()).to.equal(0);

    // enabling some guards

    let settings = defaultCandyGuardSettings();
    settings.botTax.lamports = new anchor.BN(100000000);
    settings.liveDate.date = null;
    settings.lamports.amount = new anchor.BN(1000000000);
    settings.spltoken = null;
    settings.thirdPartySigner = null;
    settings.whitelist = null;

    await program.methods.update(settings).accounts({
      candyGuard: candyGuardKey,
      authority: payer.publicKey,
    }).rpc();

    candyGuard = await program.account.candyGuard.fetch(candyGuardKey);
    // bot_tax (b001) + live_date (b010) + lamports_charge (b100)
    expect(candyGuard.features.toNumber()).to.equal(7);

    // disabling all guards

    settings.botTax = null;
    settings.liveDate = null;
    settings.lamports = null;
    settings.spltoken = null;
    settings.thirdPartySigner = null;
    settings.whitelist = null;

    await program.methods.update(settings).accounts({
      candyGuard: candyGuardKey,
      authority: payer.publicKey,
    }).rpc();

    candyGuard = await program.account.candyGuard.fetch(candyGuardKey);
    expect(candyGuard.features.toNumber()).to.equal(0);

    // enabling one again

    settings = defaultCandyGuardSettings();
    settings.botTax.lamports = new anchor.BN(100000000);
    settings.liveDate = null;
    settings.lamports = null;
    settings.spltoken = null;
    settings.thirdPartySigner = null;
    settings.whitelist = null;

    await program.methods.update(settings).accounts({
      candyGuard: candyGuardKey,
      authority: payer.publicKey,
    }).rpc();

    candyGuard = await program.account.candyGuard.fetch(candyGuardKey);
    expect(candyGuard.features.toNumber()).to.equal(1);
  });
});
