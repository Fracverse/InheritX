import { Amount, MPTAmount } from '../common';
import { BaseTransaction, GlobalFlagsInterface } from './common';
export declare enum LoanPayFlags {
    tfLoanOverpayment = 65536,
    tfLoanFullPayment = 131072,
    tfLoanLatePayment = 262144
}
export interface LoanPayFlagsInterface extends GlobalFlagsInterface {
    tfLoanOverpayment?: boolean;
    tfLoanFullPayment?: boolean;
    tfLoanLatePayment?: boolean;
}
export interface LoanPay extends BaseTransaction {
    TransactionType: 'LoanPay';
    LoanID: string;
    Amount: Amount | MPTAmount;
    Flags?: number | LoanPayFlagsInterface;
}
export declare function validateLoanPay(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanPay.d.ts.map