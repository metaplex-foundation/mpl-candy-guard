import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";
import { CandyMachine } from "../target/types/candy_machine";
import * as test from "./helpers"

describe("Mint (Candy Guard -> Candy Machine)", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());
    // candy guard base for generating the PDA address
    const candyGuardBaseKeypair = anchor.web3.Keypair.generate();
    // candy machine for the tests
    const candyMachineKeypair = anchor.web3.Keypair.generate();
    // candy guard program
    const candyGuardProgram = anchor.workspace.CandyGuard as Program<CandyGuard>;
    // candy machine program
    const candyMachineProgram = anchor.workspace.CandyMachine as Program<CandyMachine>;
    // payer for transactions
    const payer = (candyGuardProgram.provider as anchor.AnchorProvider).wallet as Wallet;

    /**
     * Creates the candy machine for the tests.
     */
    it("candy machine: initialize", async () => {
        const items = 10;
        const data = test.defaultCandyMachineSettings(items, payer.publicKey, false);
        const candyMachineKey = await test.createCandyMachine(candyMachineProgram, candyMachineKeypair, payer, data);

        let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);

        expect(candyMachine.itemsRedeemed.toNumber()).to.equal(0);
        expect(candyMachine.data.itemsAvailable.toNumber()).to.equal(items);
    });

    /**
     * Add config lines for teh test.
     */
    it("candy machine: add_config_lines", async () => {
        let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);
        const lines = [];

        for (let i = 0; i < candyMachine.data.itemsAvailable.toNumber(); i++) {
            const line = JSON.parse(`{\
                "name": "#${i + 1}",\
                "uri": "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"\
            }`);

            lines[i] = line;
        }

        await test.addConfigLines(candyMachineProgram, lines, candyMachineKeypair, payer);
    });

    /**
     * Add a collection mint.
     */
     it("candy machine: add_collection", async () => {
        await test.addCollection(candyMachineProgram, candyMachineKeypair, test.COLLECTION_MINT_ID, payer);
        let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);
        expect(candyMachine.collectionMint).to.not.equal(null);
    });

    /**
     * Minting from the candy machine without the guard.
     */
    it("candy machine: mint (without-guard)", async () => {
        await test.mintFromCandyMachine(candyMachineProgram, candyMachineKeypair, payer, test.COLLECTION_MINT_ID);
    });

    /**
     * Creates the candy guard for tests.
     */
    it("candy guard: initialize", async () => {
        const candyGuardKey = await test.createCandyGuard(candyGuardProgram, candyGuardBaseKeypair, payer)

        let candyGuard = await candyGuardProgram.account.candyGuard.fetch(candyGuardKey);
        // all guards are disable at the start
        expect(candyGuard.features.toNumber()).to.equal(0);

        // enabling some guards

        let settings = test.defaultCandyGuardSettings();
        settings.botTax.lamports = new anchor.BN(100000000);
        settings.botTax.lastInstruction = true;
        settings.liveDate = null;
        settings.lamports.amount = new anchor.BN(1000000000);
        settings.spltoken = null;
        settings.thirdPartySigner = null;
        settings.whitelist = null;

        await candyGuardProgram.methods.update(settings).accounts({
            candyGuard: candyGuardKey,
            authority: payer.publicKey,
        }).rpc();

        candyGuard = await candyGuardProgram.account.candyGuard.fetch(candyGuardKey);
        // bot_tax (b001) + lamports_charge (b100)
        expect(candyGuard.features.toNumber()).to.equal(5);
    });

    /**
     * Wraps the candy machine with a candy guard. After this, minting is only possible throught the
     * candy guard.
     */
    it("candy guard: wrap", async () => {
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
     * Minting from the candy machine is not expected to succeed.
     */
    it("candy machine: mint (forbidden)", async () => {
        let failed = false;

        try {
            await test.mintFromCandyMachine(candyMachineProgram, candyMachineKeypair, payer);
        } catch {
            // ok, we are expecting to fail
            failed = true;
        }

        expect(failed).to.equal(true);
    });

    /**
     * Mints from the candy guard + bot tax.
     */
    it("candy guard: mint", async () => {
        const signature = await test.mintFromCandyGuard(
            candyGuardProgram,
            candyMachineProgram,
            candyGuardBaseKeypair,
            candyMachineKeypair,
            payer,
            null,
            test.COLLECTION_MINT_ID
        );

        console.log(signature);
    });

    /**
     * Removes the candy guard "wrapping" from the candy machine.
     */
    it("candy guard: unwrap", async () => {
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
    });

    /**
     * Mint from the candy machine authority should work again.
     */
    it("candy machine: mint", async () => {
        await test.mintFromCandyMachine(candyMachineProgram, candyMachineKeypair, payer, test.COLLECTION_MINT_ID);
    });

    /**
     * Withdraw the rent from the candy machine.
     */
    it("withdraw", async () => {
        await candyMachineProgram.methods.withdraw().accounts({
            candyMachine: candyMachineKeypair.publicKey,
            authority: payer.publicKey,
        }).rpc();
    });
});
