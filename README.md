# Metaplex Candy Guard

> 🛑 **DO NOT USE IN PRODUCTION**: This repository contain a proof-of-concept.

## Overview

The new `Candy Guard` program is designed to take away the **access control** logic from the `Candy Machine` to handle the additional mint features, while the Candy Machine program retains its core mint functionality &mdash; the creation of the NFT. This not only provides a clear separation between **access controls** and **mint logic**, it also provides a modular and flexible architecture to add or remove mint features without having to modify the Candy Machine program.

The access control on a Candy Guard is encapsulated in individuals guards representing a specific rule that needs to be satisfied, which can be enabled or disabled. For example, the live date of the mint is represented as the `LiveDate` guard. This guard is satisfied only if the transaction time is on or after the configured start time on the guard. Other guards can validate different aspects of the access control – e.g., ensuring that the user holds a specific token (token gating).

> **Note:** The Candy Guard program can only be used in combination with `Candy Machine Core` (`Candy Machine V3`) accounts.
>
> When a Candy Guard is used in combination with a Candy Machine, it becomes its mint authority and minting is only possible through the Candy Guard.

### How the program works?

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

## Accounts

The Candy Guard configuration is stored in a single account. The information regarding the guards that are enable is stored in a "hidden" section of the account to avoid unnecessary deserialization.

| Field             | Offset | Size  | Description                  |
| ----------------- | ------ | ----- | ---------------------------- |
| &mdash;           | 0      | 8     | Anchor account discriminator.
| `base`            | 8      | 32    | `PubKey` to derive the PDA key. The seed is defined by `["candy_guard", base pubkey]`. |
| `bump`            | 40     | 1     | `u8` representing the bump of the derivation. |
| `authority`       | 41     | 32    | `PubKey` of the authority address that controls the Candy Guard. |
| *hidden section*  | 73     | ~     | Hidden data section to avoid unnecessary deserialization. This section of the account is used to serialize the guards data. |
| - *features*      | 73     | 8     | Feature flags indicating which guards are serialized. If all guards are disable on the default guard set and there are no groups defined, nothing else is stored on the account. |
| - *guard set*     | 81     | ~     | (optional) A sequence of serialized guard structs. |
| - *group counter* | ~      | 4     | `u32` specifying the number of groups in use. |
| - *groups*        | ~      | ~     | (optional) A variable number of `Group` structs representing different guard sets. Each groups is defined by:  |
| -- *label*        | ~      | 6     | A sequence of serialized guard structs. |
| -- *features*     | ~      | 8     |  Feature flags indicating which guards are serialized for the group. |
| -- *guard set*    | ~      | ~     | (optional) A sequence of serialized guard structs. |

Since the number of guards enabled and groups is variable, the account size is dynamically resized during the `update` instruction to accommodate the updated configuration.