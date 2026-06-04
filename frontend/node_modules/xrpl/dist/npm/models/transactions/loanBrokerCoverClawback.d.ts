import { IssuedCurrencyAmount, MPTAmount } from '../common';
import { BaseTransaction } from './common';
export interface LoanBrokerCoverClawback extends BaseTransaction {
    TransactionType: 'LoanBrokerCoverClawback';
    LoanBrokerID?: string;
    Amount?: IssuedCurrencyAmount | MPTAmount;
}
export declare function validateLoanBrokerCoverClawback(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanBrokerCoverClawback.d.ts.map