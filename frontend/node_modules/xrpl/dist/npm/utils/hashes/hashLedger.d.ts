import { APIVersion } from '../../models';
import { LedgerEntry } from '../../models/ledger';
import { LedgerVersionMap } from '../../models/ledger/Ledger';
import { Transaction, TransactionMetadata } from '../../models/transactions';
export declare function hashSignedTx(tx: Transaction | string): string;
export declare function hashLedgerHeader(ledgerHeader: LedgerVersionMap<APIVersion>): string;
export declare function hashTxTree(transactions: Array<Transaction & {
    metaData?: TransactionMetadata;
}>): string;
export declare function hashStateTree(entries: LedgerEntry[]): string;
declare function hashLedger(ledger: LedgerVersionMap<APIVersion>, options?: {
    computeTreeHashes?: boolean;
}): string;
export default hashLedger;
//# sourceMappingURL=hashLedger.d.ts.map