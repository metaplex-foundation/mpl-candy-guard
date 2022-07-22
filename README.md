# Candy Guard Program (PoC)

A program to handle additional mint features, while the `Candy Machine` program retain the core mint functionality. The main purpose of the `Candy Guard` program is to hold the configuration of mint guards and apply them before a user can mint from a candy machine&mdash;if all enabled guard tests are valid, the mint transaction is forwarded to the candy machine.