import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyMachine } from "../target/types/candy_machine";
import * as test from "./helpers"

describe("Collection", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());
    // candy machine for the tests
    const keypair = anchor.web3.Keypair.generate();
    // candy machine program
    const program = anchor.workspace.CandyMachine as Program<CandyMachine>;
    // payer for transactions
    const payer = (program.provider as anchor.AnchorProvider).wallet as Wallet;

    /**
     * Initializes a candy machine.
     */
    it("initialize", async () => {
        const items = 10;
        const data = test.defaultCandyMachineSettings(items, payer.publicKey, false);

        await test.createCandyMachine(program, keypair, payer, data);

        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);

        expect(candyMachine.itemsRedeemed.toNumber()).to.equal(0);
        expect(candyMachine.data.itemsAvailable.toNumber()).to.equal(items);
    });

    /**
     * Adds config lines to the candy machine.
     */
    it("add_config_lines", async () => {
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        const lines = [];

        for (let i = 0; i < candyMachine.data.itemsAvailable.toNumber(); i++) {
            const line = JSON.parse(`{\
                "name": "NFT #${i + 1}",\
                "uri": "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"\
            }`);

            lines[i] = line;
        }

        await program.methods.addConfigLines(0, lines).accounts({
            candyMachine: keypair.publicKey,
            authority: payer.publicKey,
        }).rpc();
    });

    /**
     * Add a collection mint.
     */
    it("add_collection", async () => {
        await test.addCollection(program, keypair, test.COLLECTION_MINT_ID, payer);
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        expect(candyMachine.collectionMint).to.not.equal(null);
    });

    /**
     * Remove a collection mint.
     */
    it("remove_collection", async () => {
        await test.removeCollection(program, keypair, test.COLLECTION_MINT_ID, payer);
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        expect(candyMachine.collectionMint).to.equal(null);
    });

    /**
     * Add a collection mint.
     */
    it("add_collection (for minting)", async () => {
        await test.addCollection(program, keypair, test.COLLECTION_MINT_ID, payer);
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        expect(candyMachine.collectionMint).to.not.equal(null);
    });

    /**
     * Mint an item from the candy machine.
     */
    it("mint", async () => {
        const signature = await test.mintFromCandyMachine(program, keypair, payer, test.COLLECTION_MINT_ID);
        console.log(signature);
    });

    /**
     * Withdraw the rent from the candy machine.
     */
    it("withdraw", async () => {
        await program.methods.withdraw().accounts({
            candyMachine: keypair.publicKey,
            authority: payer.publicKey,
        }).rpc();
    });
});
