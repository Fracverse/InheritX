import { AccountRootFlagsInterface } from '../ledger/AccountRoot';
import type { Transaction } from '../transactions/transaction';
export declare function parseAccountRootFlags(flags: number): AccountRootFlagsInterface;
export declare function setTransactionFlagsToNumber(tx: Transaction): void;
export declare function convertTxFlagsToNumber(tx: Transaction): number;
export declare function parseTransactionFlags(tx: Transaction): object;
//# sourceMappingURL=flags.d.ts.map