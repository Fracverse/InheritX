import { BaseTransaction } from './common';
export interface LoanDelete extends BaseTransaction {
    TransactionType: 'LoanDelete';
    LoanID: string;
}
export declare function validateLoanDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanDelete.d.ts.map