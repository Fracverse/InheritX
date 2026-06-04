import { Amount, MPTAmount } from '../common';
import { BaseTransaction, Account } from './common';
export interface LoanBrokerCoverWithdraw extends BaseTransaction {
    TransactionType: 'LoanBrokerCoverWithdraw';
    LoanBrokerID: string;
    Amount: Amount | MPTAmount;
    Destination?: Account;
    DestinationTag?: number;
}
export declare function validateLoanBrokerCoverWithdraw(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanBrokerCoverWithdraw.d.ts.map