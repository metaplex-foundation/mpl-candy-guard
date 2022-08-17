import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { SystemProgram, Keypair, PublicKey } from '@solana/web3.js';
import {
    MintLayout,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    mintTo
} from '@solana/spl-token';
import { CandyGuard } from "../target/types/candy_guard";
import { CandyMachine } from "../target/types/candy_machine";

// token metadata program
export const METAPLEX_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

/*
 * --- Candy Guard helper functions
 */

export async function createCandyGuard(
    program: Program<CandyGuard>,
    base: Keypair,
    payer: Wallet): Promise<PublicKey> {
    // candy guard pda
    const pda = await getCandyGuardPDA(program, base);

    await program.methods
        .initialize()
        .accounts({
            candyGuard: pda,
            base: base.publicKey,
            authority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: SystemProgram.programId,
        })
        .signers([base])
        .rpc();

    return pda;
}

/**
 * Helper function to create the candy guard struct. In most cases these values will be
 * replaced in the test methods.
 */
export function defaultCandyGuardSettings() {
    return JSON.parse('{\
        "botTax": {\
          "lamports": 0\
        },\
        "liveDate": {\
          "date": 0\
        },\
        "lamportsCharge": {\
          "amount": 0\
        },\
        "spltokenCharge": {\
            "amount": 0,\
            "tokenMint": null\
        },\
        "whitelist": {\
            "mint": null,\
            "presale": false,\
            "discountPrice": null,\
            "mode": "burnEveryTime"\
        }\
      }');
}

export async function getCandyGuardPDA(program: Program<CandyGuard>, base: Keypair): Promise<PublicKey> {
    return await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('candy_guard'), base.publicKey.toBuffer()],
        program.programId,
    ).then(result => { return result[0]; });
}

export async function mintFromCandyGuard(
    candyGuardProgram: Program<CandyGuard>,
    candyMachineProgram: Program<CandyMachine>,
    candyGuardBaseKeypair: Keypair,
    candyMachineKeypair: Keypair,
    payer: Wallet): Promise<string> {
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

    return await candyGuardProgram.methods
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
}

/*
 * --- Candy Machine helper functions
 */

const MAX_NAME_LENGTH = 32;
const MAX_URI_LENGTH = 200;
const MAX_SYMBOL_LENGTH = 10;
const MAX_CREATOR_LEN = 32 + 1 + 1;
const MAX_CREATOR_LIMIT = 5;

export const HIDDEN_SECTION = 8               // discriminator
    + 8                                       // features
    + 32                                      // authority
    + 32                                      // wallet
    + 33                                      // (optional) collection
    + 8                                       // items redeemed
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

export const CONFIG_NAME_LENGTH = 10;
export const CONFIG_URI_LENGTH = 50;

export function getCandyMachineSpace(items: number): number {
    return HIDDEN_SECTION
        + 4
        + items * (CONFIG_NAME_LENGTH + CONFIG_URI_LENGTH)
        + 4
        + (Math.floor(items / 8) + 1)
        + 4
        + items * 4;
}

export async function getCandyMachinePDA(program: Program<CandyMachine>, base: Keypair): Promise<PublicKey> {
    return await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('candy_machine'), base.publicKey.toBuffer()],
        program.programId,
    ).then(result => { return result[0]; });
}

/**
 * Helper function to create the candy machine data struct. In most cases these values will be
 * replaced in the test methods.
 */
export function defaultCandyMachineSettings(items: number, creator: PublicKey) {
    const settings = JSON.parse(`{\
        "itemsAvailable": ${items},\
        "symbol": "CANDYGUARD",\
        "sellerFeeBasisPoints": 500,\
        "maxSupply": 0,\
        "isMutable": true,\
        "retainAuthority": true,\
        "creators": [{\
            "address": "${creator}",\
            "percentageShare": 100\
        }],\
        "configLineSettings": {\
            "prefixName": "CandyGuard ",\
            "nameLength": ${CONFIG_NAME_LENGTH},\
            "prefixUri": \"https://arweave.net/\",\
            "uriLength": ${CONFIG_URI_LENGTH}\
        },\
        "hiddenSettings": null\
    }`);
    settings.itemsAvailable = new anchor.BN(items);
    settings.sellerFeeBasisPoints = new anchor.BN(500);
    settings.maxSupply = new anchor.BN(0);
    settings.configLineSettings.nameLength = new anchor.BN(CONFIG_NAME_LENGTH);
    settings.configLineSettings.uriLength = new anchor.BN(CONFIG_URI_LENGTH);
    settings.creators[0].address = creator;

    return settings;
}

export async function createCandyMachine(program: Program<CandyMachine>,
    base: Keypair,
    payer: Wallet,
    data: any): Promise<PublicKey> {
    // candy machine pda
    await program.methods
        .initialize(data)
        .accounts({
            candyMachine: base.publicKey,
            wallet: payer.publicKey,
            authority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .preInstructions([anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: base.publicKey,
            lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
                getCandyMachineSpace(data.itemsAvailable),
            ),
            space: getCandyMachineSpace(data.itemsAvailable),
            programId: program.programId,
        })])
        .signers([base])
        .rpc();

    return base.publicKey;
}

export async function mintFromCandyMachine(program: Program<CandyMachine>, base: Keypair, payer: Wallet): Promise<string> {
    // mint address
    const mint = anchor.web3.Keypair.generate();
    // creator PDA
    const [candyMachineCreator, bump] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('candy_machine'), base.publicKey.toBuffer()],
        program.programId,
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

    return await program.methods
        .mint(bump)
        .accounts({
            candyMachine: base.publicKey,
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
}

export async function addConfigLines(program: Program<CandyMachine>, lines: any, base: Keypair, payer: Wallet) {
    let candyMachine = await program.account.candyMachine.fetch(base.publicKey);
    const available = candyMachine.data.itemsAvailable.toNumber();
    let start = 0;

    while (start < available) {
        const limit = Math.min(available - start, 10);

        await program.methods.addConfigLines(start, lines.slice(start, limit)).accounts({
            candyMachine: base.publicKey,
            authority: payer.publicKey,
        }).rpc();

        start = start + limit;
    }
}
