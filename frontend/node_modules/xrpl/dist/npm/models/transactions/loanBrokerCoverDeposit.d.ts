import { Amount, MPTAmount } from '../common';
import { BaseTransaction } from './common';
export interface LoanBrokerCoverDeposit extends BaseTransaction {
    TransactionType: 'LoanBrokerCoverDeposit';
    LoanBrokerID: string;
    Amount: Amount | MPTAmount;
}
export declare function validateLoanBrokerCoverDeposit(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanBrokerCoverDeposit.d.ts.map