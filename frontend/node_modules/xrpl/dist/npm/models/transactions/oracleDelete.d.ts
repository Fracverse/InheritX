import { BaseTransaction } from './common';
export interface OracleDelete extends BaseTransaction {
    TransactionType: 'OracleDelete';
    OracleDocumentID: number;
}
export declare function validateOracleDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=oracleDelete.d.ts.map