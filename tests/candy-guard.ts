import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";

describe("Candy Guard", () => {
  // configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  // candy guard for the tests
  const keypair = anchor.web3.Keypair.generate();
  // candy guard program
  const program = anchor.workspace.CandyGuard as Program<CandyGuard>;
  // payer of the transactions
  const payer = (program.provider as anchor.AnchorProvider).wallet;

  it("initialize", async () => {
    await program.methods
      .initialize()
      .accounts({
        candyGuard: keypair.publicKey,
        authority: payer.publicKey,
        payer: payer.publicKey,
      })
      .signers([keypair])
      .rpc();

    let candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);

    expect(candy_guard.features.toNumber()).to.equal(0);
  });

  it("update", async () => {
    let candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    expect(candy_guard.features.toNumber()).to.equal(0);

    const settings = JSON.parse('{\
      "botTax": {\
        "lamports": 100000000\
      },\
      "liveDate": {\
        "date": 1657669708\
      },\
      "lamportsCharge": {\
        "amount": 1000000000\
      },\
      "spltokenCharge": null,\
      "whitelist": null\
    }');

    settings.botTax.lamports = new anchor.BN(100000000);
    settings.liveDate.date = null;
    settings.lamportsCharge.amount = new anchor.BN(1000000000);

    await program.methods.update(settings).accounts({
      candyGuard: keypair.publicKey,
      authority: payer.publicKey,
    }).rpc();

    candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    // bot_tax (1) + live_date (2) + lamports_charge (8)
    expect(candy_guard.features.toNumber()).to.equal(11);
  });
});
