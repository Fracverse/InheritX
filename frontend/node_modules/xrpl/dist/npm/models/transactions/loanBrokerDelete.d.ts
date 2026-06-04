import { BaseTransaction } from './common';
export interface LoanBrokerDelete extends BaseTransaction {
    TransactionType: 'LoanBrokerDelete';
    LoanBrokerID: string;
}
export declare function validateLoanBrokerDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanBrokerDelete.d.ts.map