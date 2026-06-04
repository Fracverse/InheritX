import { BaseTransaction } from './common';
export interface MPTokenIssuanceDestroy extends BaseTransaction {
    TransactionType: 'MPTokenIssuanceDestroy';
    MPTokenIssuanceID: string;
}
export declare function validateMPTokenIssuanceDestroy(tx: Record<string, unknown>): void;
//# sourceMappingURL=MPTokenIssuanceDestroy.d.ts.map