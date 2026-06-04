import type { Transaction, TransactionVersion, VersionedTransaction } from '@solana/web3.js';
export type SupportedTransactionVersions = ReadonlySet<TransactionVersion> | null | undefined;
export type TransactionOrVersionedTransaction<S extends SupportedTransactionVersions> = S extends null | undefined ? Transaction : Transaction | VersionedTransaction;
export declare function isVersionedTransaction(transaction: Transaction | VersionedTransaction): transaction is VersionedTransaction;
//# sourceMappingURL=transaction.d.ts.map