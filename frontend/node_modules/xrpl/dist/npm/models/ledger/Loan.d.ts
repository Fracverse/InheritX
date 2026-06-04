import { Account, XRPLNumber } from '../transactions/common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface Loan extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Loan';
    Flags: number;
    LoanSequence: number;
    OwnerNode: string;
    LoanBrokerNode: string;
    LoanBrokerID: string;
    Borrower: Account;
    LoanOriginationFee?: XRPLNumber;
    LoanServiceFee?: XRPLNumber;
    LatePaymentFee?: XRPLNumber;
    ClosePaymentFee?: XRPLNumber;
    OverpaymentFee?: XRPLNumber;
    InterestRate?: number;
    LateInterestRate?: number;
    CloseInterestRate?: number;
    OverpaymentInterestRate?: number;
    StartDate: number;
    PaymentInterval: number;
    GracePeriod: number;
    PreviousPaymentDueDate?: number;
    NextPaymentDueDate: number;
    PaymentRemaining: number;
    TotalValueOutstanding: XRPLNumber;
    PrincipalOutstanding: XRPLNumber;
    ManagementFeeOutstanding?: XRPLNumber;
    PeriodicPayment: XRPLNumber;
    LoanScale?: number;
}
export declare enum LoanFlags {
    lsfLoanDefault = 65536,
    lsfLoanImpaired = 131072,
    lsfLoanOverpayment = 262144
}
//# sourceMappingURL=Loan.d.ts.map