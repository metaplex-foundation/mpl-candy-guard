export * from './asserts';
export * from './constants';
export * from './errors';
export * from './helper';

import { CandyMachineData } from '@metaplex-foundation/mpl-candy-machine-core';
import { Keypair, PublicKey } from '@solana/web3.js';
import { HIDDEN_SECTION } from './constants';

export async function getCandyGuardPDA(programId: PublicKey, base: Keypair): Promise<PublicKey> {
  return await PublicKey.findProgramAddress(
    [Buffer.from('candy_guard'), base.publicKey.toBuffer()],
    programId,
  ).then((result) => {
    return result[0];
  });
}

export function getCandyMachineSpace(data: CandyMachineData): number {
  if (data.configLineSettings == null) {
    return HIDDEN_SECTION;
  } else {
    const items = parseInt(data.itemsAvailable.toString());
    return (
      HIDDEN_SECTION +
      4 +
      items * (data.configLineSettings.nameLength + data.configLineSettings.uriLength) +
      4 +
      (Math.floor(items / 8) + 1) +
      4 +
      items * 4
    );
  }
}
