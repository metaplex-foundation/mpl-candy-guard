import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyMachine } from "../target/types/candy_machine";
import { defaultCandyMachineSettings, createCandyMachine, mintFromCandyMachine } from "./helpers"

describe("Candy Machine", () => {
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
        const data = defaultCandyMachineSettings(items, payer.publicKey);
        const candyMachineKey = await createCandyMachine(program, keypair, payer, data);

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
     * Mint an item from the candy machine.
     */
    it("mint", async () => {
        const signature = await mintFromCandyMachine(program, keypair, payer);
        console.log(signature);
    });
});
