# Metaplex Candy Guard

> 🛑 **DO NOT USE IN PRODUCTION**: This repository contain a proof-of-concept.

## Overview

The new `Candy Guard` program is designed to take away the **access control** logic from the `Candy Machine` to handle the additional mint features, while the Candy Machine program retains its core mint functionality &mdash; the creation of the NFT. This not only provides a clear separation between **access controls** and **mint logic**, it also provides a modular and flexible architecture to add or remove mint features without having to modify the Candy Machine program.

The access control on a Candy Guard is encapsulated in individuals guards representing a specific rule that needs to be satisfied, which can be enabled or disabled. For example, the live date of the mint is represented as the `LiveDate` guard. This guard is satisfied only if the transaction time is on or after the configured start time on the guard. Other guards can validate different aspects of the access control – e.g., ensuring that the user holds a specific token (token gating).

> **Note:**
> The Candy Guard program can only be used in combination with `Candy Machine Core` (`Candy Machine V3`) accounts.
>
> When a Candy Guard is used in combination with a Candy Machine, it becomes its mint authority and minting is only possible through the Candy Guard.

### How the program works?

![image](https://user-images.githubusercontent.com/729235/192137030-4ee8affa-ee6d-4a56-a273-e17ce64723f9.png)

The main purpose of the Candy Guard program is to hold the configuration of mint **guards** and apply them before a user can mint from a candy machine. If all enabled guard conditions are valid, the mint transaction is forwarded to the Candy Machine.

When a mint transaction is received, the program performs the following steps:

1. Validates the transaction against all enabled guards.
    - If any of the guards fail at this point, the transaction is subject to the `BotTax` (when the `BotTax` guard is enabled) and the transaction is then aborted.
2. After evaluating that all guards are valid, it invokes the `pre_actions` function on each guard. This function is responsible to perform any action **before** the mint (e.g., take payment for the mint).
3. Then the transaction is forwarded to the Candy Machine program to mint the NFT. 
4. Finally, it invokes the `post_actions` function on each enabled guard. This function is responsible to perform any action **after** the mint (e.g., freeze the NFT, change the update authority).

A **guard** is a modular piece of code that can be easily added to the Candy Guard program, providing great flexibility and simplicity to support specific features without having to modify directly the Candy Machine program. Adding new guards is supported by conforming to specific interfaces, with changes isolated to the individual guard – e.g., each guard can be created and modified in isolation. This architecture also provides the flexibility to enable/disable guards without requiring code changes, as each guard has an enable/disable "switch".

### Guards

The Candy Guard program contains a set of core access control guards that can be enabled/disabled:

- `BotTax`: configurable tax (amount) to charge invalid transactions
- `Lamports`: set the price of the mint in SOL
- `SplToken`: set the price of the mint in spl-token amount
- `LiveDate`: controls when the mint is allowed
- `ThirdPartySigner`: requires an additional signer on the transaction 
- `Whitelist`: allows the creation of a whitelist with specific mint date and discount price
- `Gatekeeper`: captcha integration
- `EndSettings`: determines a rule to end the mint period based on date or amount
- `AllowList`: uses a wallet address list to determine who is allowed to mint
- `MintLimit`: enforces mint limits on wallet addresses
- `NftPayment`: requires an NFT as a payment method

## Account

The Candy Guard configuration is stored in a single account. The information regarding the guards that are enable is stored in a "hidden" section of the account to avoid unnecessary deserialization.

| Field             | Offset | Size  | Description                  |
| ----------------- | ------ | ----- | ---------------------------- |
| &mdash;           | 0      | 8     | Anchor account discriminator.
| `base`            | 8      | 32    | `PubKey` to derive the PDA key. The seed is defined by `["candy_guard", base pubkey]`. |
| `bump`            | 40     | 1     | `u8` representing the bump of the derivation. |
| `authority`       | 41     | 32    | `PubKey` of the authority address that controls the Candy Guard. |
| *hidden section*  | 73     | ~     | Hidden data section to avoid unnecessary deserialization. This section of the account is used to serialize the guards data. |
| - *features*      | 73     | 8     | Feature flags indicating which guards are serialized. |
| - *guard set*     | 81     | ~     | (optional) A sequence of serialized guard structs. |
| - *group counter* | ~      | 4     | `u32` specifying the number of groups in use. |
| - *groups*        | ~      | ~     | (optional) A variable number of `Group` structs representing different guard sets. Each groups is defined by:  |
| -- *label*        | ~      | 6     | A sequence of serialized guard structs. |
| -- *features*     | ~      | 8     |  Feature flags indicating which guards are serialized for the group. |
| -- *guard set*    | ~      | ~     | (optional) A sequence of serialized guard structs. |

Since the number of guards enabled and groups is variable, the account size is dynamically resized during the `update` instruction to accommodate the updated configuration.

## Instructions

### 📄 `initialize`

This instruction creates and initializes a new `CandyGuard` account.

<details>
  <summary>Accounts</summary>

| Name                 | Writable | Signer | Description |
| ---------------------| :------: | :----: | ----------- |
| `candy_guard`        | ✅       |        | The `CandyGuard` account PDA key. The PDA is derived using the seed `["candy_guard", base pubkey]`. |
| `base`               |          | ✅     | Base public key for the PDA derivation. |
| `authority`          |          |        | Public key of the candy guard authority. |
| `payer`              |          | ✅     | Payer of the transaction. |
| `system_program`     |          |        | `SystemProgram` account. |
</details>

<details>
  <summary>Arguments</summary>
  
| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `data`                        | 0      | ~    | `CandyGuardData` object. |
</details>


### 📄 `mint`

This instruction mints an NFT from a Candy Machine "wrapped" by a Candy Guard. Only when the transaction is succesfully validated, it is forwarded to the Candy Machine.

<details>
  <summary>Accounts</summary>

| Name                          | Writable | Signer | Description |
| ----------------------------- | :------: | :----: | ----------- |
| `candy_guard`                 |          |        | The `CandyGuard` account PDA key. The PDA is derived using the seed `["candy_guard", base pubkey]`. |
| `candy_machine_program`       |          |        | `CandyMachine` program ID. |
| `candy_machine`               | ✅       |        | The `CandyMachine` account. |
| `candy_machine_authority_pda` | ✅       |        | Authority PDA key (seeds `["candy_machine", candy_machine pubkey]`). |
| `payer`                       | ✅       | ✅     | Payer of the transaction. |
| `nft_metadata`                | ✅       |        | Metadata account of the NFT. |
| `nft_mint`                    | ✅       |        | Mint account for the NFT. The account should be created before executing the instruction. |
| `nft_mint_authority`          |          | ✅     | Mint authority of the NFT. |
| `nft_master_edition`          | ✅       |        | Master Edition account of the NFT. |
| `collection_authority_record` |          |        | Authority Record PDA of the collection. |
| `collection_mint`             |          |        | Mint account of the collection. |
| `collection_metadata`         | ✅       |        | Metadata account of the collection. |
| `collection_master_edition`   |          |        | Master Edition account of the collection. |
| `collection_update_authority` |          |        | Update authority of the collection. |
| `token_metadata_program`      |          |        | Metaplex `TokenMetadata` program ID. |
| `token_program`               |          |        | `spl-token` program ID. |
| `system_program`              |          |        | `SystemProgram` account. |
| `rent`                        |          |        | `Rent` account. |
| `recent_slothashes`           |          |        | `SlotHashes` account. |
| `instruction_sysvar_account`  |          |        | `Sysvar1nstructions` account. |
| *remaining accounts*          |          |        | (optional) A list of optional accounts required by individual guards. |
</details>

<details>
  <summary>Arguments</summary>
  
| Argument        | Offset | Size | Description               |
| --------------- | ------ | ---- | ------------------------- |
| `mint_args`     | 0      | ~    | `[u8]` representing arguments for guards; an empty `[u8]` if there are no arguments. |
| `label`         | ~      | 6    | (optional) `string` representing the group label to use for validation of guards. |
</details>


### 📄 `unwrap`

This instruction removes a Candy Guard from a Candy Machine, setting the mint authority of the Candy Machine to be the Candy Machine authority. The Candy Gard `public key` must match the Candy Machine `mint_authority` for this instruction to succeed.

<details>
  <summary>Accounts</summary>

| Name                      | Writable | Signer | Description |
| ------------------------- | :------: | :----: | ----------- |
| `candy_guard`             |          |        | The `CandyGuard` account PDA key. |
| `authority`               |          | ✅     | Public key of the `candy_guard` authority. |
| `candy_machine`           | ✅       |        | The `CandyMachine` account. |
| `candy_machine_authority` |          | ✅     | Public key of the `candy_machine` authority. |
| `candy_machine_program`   |          |        | `CandyMachine` program ID. |
</details>

<details>
  <summary>Arguments</summary>
  
None.
</details>


### 📄 `update`

This instruction updates the Candy Guard configuration. Given that there is a flexible number of guards and groups that can be present, this instruction will resize the account accordingly, either increasing or decreasing the account size. Therefore, there will be either a charge for rent or a withdraw of rent lamports. 

<details>
  <summary>Accounts</summary>

| Name             | Writable | Signer | Description |
| ---------------- | :------: | :----: | ----------- |
| `candy_guard`    | ✅       |        | The `CandyGuard` account PDA key. |
| `authority`      |          |        | Public key of the `candy_guard` authority. |
| `payer`          |          | ✅     | Payer of the transaction. |
| `system_program` |          |        | `SystemProgram` account. |
</details>

<details>
  <summary>Arguments</summary>
  
| Argument                      | Offset | Size | Description               |
| ----------------------------- | ------ | ---- | ------------------------- |
| `data`                        | 0      | ~    | `CandyGuardData` object. |
</details>


### 📄 `withdraw`

This instruction withdraws the rent lamports from the account and closes it. After executing this instruction, the Candy Guard account will not be operational.

<details>
  <summary>Accounts</summary>

| Name          | Writable | Signer | Description |
| --------------| :------: | :----: | ----------- |
| `candy_guard` | ✅       |        | The `CandyGuard` account. |
| `authority`   | ✅       | ✅     | Public key of the `candy_guard` authority. |
</details>

<details>
  <summary>Arguments</summary>
  
None.
</details>


### 📄 `wrap`

This instruction adds a Candy Guard to a Candy Machine. After the guard is added, minting is only allowed through the Candy Guard.

<details>
  <summary>Accounts</summary>

| Name                      | Writable | Signer | Description |
| ------------------------- | :------: | :----: | ----------- |
| `candy_guard`             |          |        | The `CandyGuard` account PDA key. |
| `authority`               |          | ✅     | Public key of the `candy_guard` authority. |
| `candy_machine`           | ✅       |        | The `CandyMachine` account. |
| `candy_machine_authority` |          | ✅     | Public key of the `candy_machine` authority. |
| `candy_machine_program`   |          |        | `CandyMachine` program ID. |
</details>

<details>
  <summary>Arguments</summary>
  
None.
</details>
