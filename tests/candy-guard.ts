import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";

describe("candy-guard", () => {
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

    const settings = JSON.parse("{\
      \"botTax\": {\
        \"lamports\": 100000000\
      },\
      \"liveDate\": {\
        \"date\": 1657669708\
      },\
      \"whitelist\": null\
    }");
    /*
    const settings = JSON.parse("{\
      \"botTax\": {\
        \"lamports\": 100000000\
      },\
      \"liveDate\": null,\
      \"whitelist\": null\
    }");
    */
    settings.botTax.lamports = new anchor.BN(100000000);
    settings.liveDate.date = null;

    await program.methods.update(settings).accounts({
      candyGuard: keypair.publicKey,
      authority: payer.publicKey,
    }).rpc();

    candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    expect(candy_guard.features.toNumber()).to.equal(3);
  });

  it("Mint!", async () => {
    let candy_guard = await program.account.candyGuard.fetch(keypair.publicKey);
    expect(candy_guard.features.toNumber()).to.equal(3);

    const signature = await program.methods.mint().accounts({
      candyGuard: keypair.publicKey,
      payer: payer.publicKey,
    }).rpc();

    console.log(signature);
  });
});
