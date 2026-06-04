import { Batch } from '../models';
import type { Wallet } from './index';
export declare function signMultiBatch(wallet: Wallet, transaction: Batch, opts?: {
    batchAccount?: string;
    multisign?: boolean | string;
}): void;
export declare function combineBatchSigners(transactions: Array<Batch | string>): string;
//# sourceMappingURL=batchSigner.d.ts.map