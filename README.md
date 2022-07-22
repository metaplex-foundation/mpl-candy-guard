# **🚨 EXPERIMENTAL**

> This repository contain a proof-of-concept. 🛑 **DO NOT USE IN PRODUCTION**.

## Candy Guard Program (PoC)

A program to handle additional mint features, while the `Candy Machine` program retain the core mint functionality. The main purpose of the `Candy Guard` program is to hold the configuration of mint guards and apply them before a user can mint from a candy machine &mdash; if all enabled guard tests are valid, the mint transaction is forwarded to the candy machine.

## Running

To build the programs (from the root of the repository):
```
anchor build
```

Once the programs are build, you will need to update their program id. To deploy the programs, run:
```
anchor deploy
```

There are a few tests included, which can be run with:
```
achor tests
```