import { Signer } from '../common';
import { BaseTransaction, GlobalFlagsInterface } from './common';
import type { SubmittableTransaction } from './transaction';
export declare enum BatchFlags {
    tfAllOrNothing = 65536,
    tfOnlyOne = 131072,
    tfUntilFailure = 262144,
    tfIndependent = 524288
}
export interface BatchFlagsInterface extends GlobalFlagsInterface {
    tfAllOrNothing?: boolean;
    tfOnlyOne?: boolean;
    tfUntilFailure?: boolean;
    tfIndependent?: boolean;
}
export interface BatchSigner {
    BatchSigner: {
        Account: string;
        SigningPubKey?: string;
        TxnSignature?: string;
        Signers?: Signer[];
    };
}
export interface Batch extends BaseTransaction {
    TransactionType: 'Batch';
    BatchSigners?: BatchSigner[];
    RawTransactions: Array<{
        RawTransaction: SubmittableTransaction;
    }>;
}
export declare function validateBatch(tx: Record<string, unknown>): void;
//# sourceMappingURL=batch.d.ts.map