import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyGuard } from "../target/types/candy_guard";
import { CandyMachine } from "../target/types/candy_machine";
import { SystemProgram } from '@solana/web3.js';
import {
    MintLayout,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

export const MAX_NAME_LENGTH = 32;
export const MAX_URI_LENGTH = 200;
export const MAX_SYMBOL_LENGTH = 10;
export const MAX_CREATOR_LEN = 32 + 1 + 1;
export const MAX_CREATOR_LIMIT = 5;

describe("Mint CPI", () => {
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
    // payer of the transactions
    const payer = (candyGuardProgram.provider as anchor.AnchorProvider).wallet;
    // token metadata program
    const METAPLEX_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    it("initialize (candy machine)", async () => {
        const HIDDEN_SECTION = 8                      // discriminator
            + 32                                      // authority
            + 32                                      // wallet
            + 33                                      // (optional) token mint
            + 8                                       // items redeemed
            + 8                                       // price
            + 8                                       // items available
            + 4 + MAX_SYMBOL_LENGTH                   // u32 len + symbol
            + 2                                       // seller fee basis points
            + 8                                       // max supply
            + 1                                       // is mutable
            + 1                                       // retain authority
            + 4 + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN // u32 len + creators vec
            + 4 + MAX_NAME_LENGTH                     // u32 len +prefix_name length (config line setting)
            + 4                                       // name length
            + 4 + MAX_URI_LENGTH                      // u32 len +prefix_uri length
            + 4                                       // uri length
            + 1                                       // option (hidden setting)
            + 4 + MAX_NAME_LENGTH                     // u32 len +name length
            + 4 + MAX_URI_LENGTH                      // u32 len +uri length
            + 32;                                     // hash

        // number of items
        const items = 10;

        const CANDY_SPACE = HIDDEN_SECTION
            + 4
            + items * (10 + 43)
            + 4
            + (Math.floor(items / 8) + 1)
            + 4
            + items * 4

        const data = JSON.parse('{\
            "price": 1,\
            "itemsAvailable": 10,\
            "symbol": "CANDYGUARD",\
            "sellerFeeBasisPoints": 500,\
            "maxSupply": 0,\
            "isMutable": true,\
            "retainAuthority": true,\
            "creators": [{\
                "address": "N4f6zftYsuu4yT7icsjLwh4i6pB1zvvKbseHj2NmSQw",\
                "percentageShare": 100\
            }],\
            "configLineSettings": {\
                "prefixName": "CandyGuard ",\
                "nameLength": 10,\
                "prefixUri": \"https://arweave.net/\",\
                "uriLength": 43\
            },\
            "hiddenSettings": null\
        }');

        data.price = new BN(1);
        data.itemsAvailable = new BN(items);
        data.sellerFeeBasisPoints = new BN(500);
        data.maxSupply = new BN(0);
        data.configLineSettings.nameLength = new BN(10);
        data.configLineSettings.uriLength = new BN(43);
        data.creators[0].address = payer.publicKey;

        await candyMachineProgram.methods
            .initialize(data)
            .accounts({
                candyMachine: candyMachineKeypair.publicKey,
                wallet: payer.publicKey,
                authority: payer.publicKey,
                payer: payer.publicKey,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: candyMachineKeypair.publicKey,
                lamports: await candyMachineProgram.provider.connection.getMinimumBalanceForRentExemption(
                    CANDY_SPACE,
                ),
                space: CANDY_SPACE,
                programId: candyMachineProgram.programId,
            })])
            .signers([candyMachineKeypair])
            .rpc();

        let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);

        expect(candyMachine.itemsRedeemed.toNumber()).to.equal(0);
        expect(candyMachine.data.price.toNumber()).to.equal(1);
        expect(candyMachine.data.itemsAvailable.toNumber()).to.equal(items);
    });

    it("add_config_lines", async () => {
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

    it("mint (without-guard)", async () => {
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), candyMachineKeypair.publicKey.toBuffer()],
            candyMachineProgram.programId,
        );
        // associated token address
        const [associatedToken,] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        const signature = await candyMachineProgram.methods
            .mint(bump)
            .accounts({
                candyMachine: candyMachineKeypair.publicKey,
                candyMachineCreator: candyMachineCreator,
                authority: payer.publicKey,
                payer: payer.publicKey,
                metadata: metadataAddress,
                mint: mint.publicKey,
                mintAuthority: payer.publicKey,
                updateAuthority: payer.publicKey,
                masterEdition: masterEdition,
                tokenMetadataProgram: METAPLEX_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                recentSlothashes: anchor.web3.SYSVAR_SLOT_HASHES_PUBKEY,
                instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: mint.publicKey,
                lamports: await candyMachineProgram.provider.connection.getMinimumBalanceForRentExemption(
                    MintLayout.span,
                ),
                space: MintLayout.span,
                programId: TOKEN_PROGRAM_ID,
            }),
            createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey),
            createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, payer.publicKey, mint.publicKey),
            createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, [])])
            .signers([mint])
            .rpc();

        console.log(signature);
    });

    it("initialize (candy guard)", async () => {
        // candy guard PDA
        const [pda,] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_guard'), candyGuardBaseKeypair.publicKey.toBuffer()],
            candyGuardProgram.programId,
        );

        await candyGuardProgram.methods
            .initialize()
            .accounts({
                candyGuard: pda,
                base: candyGuardBaseKeypair.publicKey,
                authority: payer.publicKey,
                payer: payer.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([candyGuardBaseKeypair])
            .rpc();

        let candy_guard = await candyGuardProgram.account.candyGuard.fetch(pda);
        expect(candy_guard.features.toNumber()).to.equal(0);

        const settings = JSON.parse('{\
            "botTax": {\
                "lamports": 1000000000\
            },\
            "liveDate": {\
              "date": 0\
            },\
            "lamportsCharge": {\
                "amount": 1000000000\
            },\
            "spltokenCharge": null,\
            "whitelist": null\
          }');

        settings.botTax.lamports = new anchor.BN(1000000000);
        const date = new Date('12/25/2022 00:00:00');
        settings.liveDate.date = new BN(date.getTime());
        settings.lamportsCharge.amount = new anchor.BN(1000000000);

        await candyGuardProgram.methods.update(settings).accounts({
            candyGuard: pda,
            authority: payer.publicKey,
        }).rpc();

        candy_guard = await candyGuardProgram.account.candyGuard.fetch(pda);
        // bot_tax (1) + live_date (2) + lamports_charge (8)
        expect(candy_guard.features.toNumber()).to.equal(11);
    });

    it("wrap_candy_machine", async () => {
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

    it("mint (candy machine)", async () => {
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), candyMachineKeypair.publicKey.toBuffer()],
            candyMachineProgram.programId,
        );
        // associated token address
        const [associatedToken,] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        var failed = false;

        try {
            await candyMachineProgram.methods
                .mint(bump)
                .accounts({
                    candyMachine: candyMachineKeypair.publicKey,
                    candyMachineCreator: candyMachineCreator,
                    authority: payer.publicKey,
                    payer: payer.publicKey,
                    metadata: metadataAddress,
                    mint: mint.publicKey,
                    mintAuthority: payer.publicKey,
                    updateAuthority: payer.publicKey,
                    masterEdition: masterEdition,
                    tokenMetadataProgram: METAPLEX_PROGRAM_ID,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    recentSlothashes: anchor.web3.SYSVAR_SLOT_HASHES_PUBKEY,
                    instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
                })
                .preInstructions([anchor.web3.SystemProgram.createAccount({
                    fromPubkey: payer.publicKey,
                    newAccountPubkey: mint.publicKey,
                    lamports: await candyMachineProgram.provider.connection.getMinimumBalanceForRentExemption(
                        MintLayout.span,
                    ),
                    space: MintLayout.span,
                    programId: TOKEN_PROGRAM_ID,
                }),
                createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey),
                createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, payer.publicKey, mint.publicKey),
                createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, [])])
                .signers([mint])
                .rpc();
        } catch {
            // ok, we are expecting to fail
            failed = true;
        }

        expect(failed).to.equal(true);
    });

    it("mint (candy guard)", async () => {
        // candy guard PDA
        const [pda,] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_guard'), candyGuardBaseKeypair.publicKey.toBuffer()],
            candyGuardProgram.programId,
        );
        // candy machine
        let candyMachine = await candyMachineProgram.account.candyMachine.fetch(candyMachineKeypair.publicKey);
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), candyMachineKeypair.publicKey.toBuffer()],
            candyMachineProgram.programId,
        );
        // associated token address
        const [associatedToken,] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        const signature = await candyGuardProgram.methods
            .mint(bump)
            .accounts({
                candyGuard: pda,
                candyMachineProgram: candyMachineProgram.programId,
                candyMachine: candyMachineKeypair.publicKey,
                candyMachineCreator: candyMachineCreator,
                payer: payer.publicKey,
                wallet: candyMachine.wallet,
                metadata: metadataAddress,
                mint: mint.publicKey,
                mintAuthority: payer.publicKey,
                updateAuthority: payer.publicKey,
                masterEdition: masterEdition,
                tokenMetadataProgram: METAPLEX_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                recentSlothashes: anchor.web3.SYSVAR_SLOT_HASHES_PUBKEY,
                instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: mint.publicKey,
                lamports: await candyGuardProgram.provider.connection.getMinimumBalanceForRentExemption(
                    MintLayout.span,
                ),
                space: MintLayout.span,
                programId: TOKEN_PROGRAM_ID,
            }),
            createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey),
            createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, payer.publicKey, mint.publicKey),
            createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, [])])
            .signers([mint])
            .rpc();

        console.log(signature);
    });

    it("unwrap_candy_machine", async () => {
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

    it("mint (candy machine)", async () => {
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), candyMachineKeypair.publicKey.toBuffer()],
            candyMachineProgram.programId,
        );
        // associated token address
        const [associatedToken,] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition,] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        var failed = false;


        const signature = await candyMachineProgram.methods
            .mint(bump)
            .accounts({
                candyMachine: candyMachineKeypair.publicKey,
                candyMachineCreator: candyMachineCreator,
                authority: payer.publicKey,
                payer: payer.publicKey,
                metadata: metadataAddress,
                mint: mint.publicKey,
                mintAuthority: payer.publicKey,
                updateAuthority: payer.publicKey,
                masterEdition: masterEdition,
                tokenMetadataProgram: METAPLEX_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                recentSlothashes: anchor.web3.SYSVAR_SLOT_HASHES_PUBKEY,
                instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: mint.publicKey,
                lamports: await candyMachineProgram.provider.connection.getMinimumBalanceForRentExemption(
                    MintLayout.span,
                ),
                space: MintLayout.span,
                programId: TOKEN_PROGRAM_ID,
            }),
            createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey),
            createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, payer.publicKey, mint.publicKey),
            createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, [])])
            .signers([mint])
            .rpc();

        console.log(signature);
    });
});
