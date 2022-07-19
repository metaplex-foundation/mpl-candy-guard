import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { expect } from 'chai';
import { CandyMachine } from "../target/types/candy_machine";
import { SystemProgram } from '@solana/web3.js';
import { MintLayout, createAssociatedTokenAccountInstruction, createInitializeMintInstruction, createMintToInstruction, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, mintTo } from '@solana/spl-token';

export const MAX_NAME_LENGTH = 32;
export const MAX_URI_LENGTH = 200;
export const MAX_SYMBOL_LENGTH = 10;
export const MAX_CREATOR_LEN = 32 + 1 + 1;
export const MAX_CREATOR_LIMIT = 5;

describe("candy-machine", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());
    // candy machine for the tests
    const keypair = anchor.web3.Keypair.generate();
    // candy guard program
    const program = anchor.workspace.CandyMachine as Program<CandyMachine>;
    // payer of the transactions
    const payer = (program.provider as anchor.AnchorProvider).wallet;
    // token metadata program
    const METAPLEX_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    async function mint(user: anchor.web3.Keypair) {
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), keypair.publicKey.toBuffer()],
            program.programId,
        );
        // associated token address
        const [associatedToken, ] = await anchor.web3.PublicKey.findProgramAddress(
            [user.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress, ] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition, ] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        return await program.methods
            .mint(bump)
            .accounts({
                candyMachine: keypair.publicKey,
                candyMachineCreator: candyMachineCreator,
                payer: user.publicKey,
                wallet: candyMachine.wallet,
                metadata: metadataAddress,
                mint: mint.publicKey,
                mintAuthority: user.publicKey,
                updateAuthority: user.publicKey,
                masterEdition: masterEdition,
                tokenMetadataProgram: METAPLEX_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                recentSlothashes: anchor.web3.SYSVAR_SLOT_HASHES_PUBKEY,
                instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: user.publicKey,
                newAccountPubkey: mint.publicKey,
                lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
                    MintLayout.span,
                ),
                space: MintLayout.span,
                programId: TOKEN_PROGRAM_ID,
            }),
            createInitializeMintInstruction(mint.publicKey, 0, user.publicKey, user.publicKey),
            createAssociatedTokenAccountInstruction(user.publicKey, associatedToken, user.publicKey, mint.publicKey),
            createMintToInstruction(mint.publicKey, associatedToken, user.publicKey, 1, [])])
            .signers([mint])
            .rpc();
    }

    it("initialize", async () => {
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

        await program.methods
            .initialize(data)
            .accounts({
                candyMachine: keypair.publicKey,
                wallet: payer.publicKey,
                authority: payer.publicKey,
                payer: payer.publicKey,
                systemProgram: SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .preInstructions([anchor.web3.SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: keypair.publicKey,
                lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
                    CANDY_SPACE,
                ),
                space: CANDY_SPACE,
                programId: program.programId,
            })])
            .signers([keypair])
            .rpc();

        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);

        expect(candyMachine.itemsRedeemed.toNumber()).to.equal(0);
        expect(candyMachine.data.price.toNumber()).to.equal(1);
        expect(candyMachine.data.itemsAvailable.toNumber()).to.equal(items);
    });

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

    it("mint", async () => {
        let candyMachine = await program.account.candyMachine.fetch(keypair.publicKey);
        // mint address
        const mint = anchor.web3.Keypair.generate();
        // creator PDA
        const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), keypair.publicKey.toBuffer()],
            program.programId,
        );
        // associated token address
        const [associatedToken, ] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // metadata address
        const [metadataAddress, ] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        // master edition address
        const [masterEdition, ] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );

        const signature = await program.methods
            .mint(bump)
            .accounts({
                candyMachine: keypair.publicKey,
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
                lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
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
