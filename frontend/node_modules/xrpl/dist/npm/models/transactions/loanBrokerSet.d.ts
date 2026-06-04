import { BaseTransaction, XRPLNumber } from './common';
export interface LoanBrokerSet extends BaseTransaction {
    TransactionType: 'LoanBrokerSet';
    VaultID: string;
    LoanBrokerID?: string;
    Data?: string;
    ManagementFeeRate?: number;
    DebtMaximum?: XRPLNumber;
    CoverRateMinimum?: number;
    CoverRateLiquidation?: number;
}
export declare function validateLoanBrokerSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanBrokerSet.d.ts.map