export * from './CandyGuard';
export * from './MintCounter';
import { MintCounter } from './MintCounter';
import { CandyGuard } from './CandyGuard';
export declare const accountProviders: {
    MintCounter: typeof MintCounter;
    CandyGuard: typeof CandyGuard;
};
