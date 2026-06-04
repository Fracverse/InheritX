import { Signer } from '../common';
import { BaseTransaction, XRPLNumber, GlobalFlagsInterface, Account } from './common';
export interface LoanSet extends BaseTransaction {
    TransactionType: 'LoanSet';
    LoanBrokerID: string;
    PrincipalRequested: XRPLNumber;
    CounterpartySignature?: CounterpartySignature;
    Counterparty?: Account;
    Data?: string;
    LoanOriginationFee?: XRPLNumber;
    LoanServiceFee?: XRPLNumber;
    LatePaymentFee?: XRPLNumber;
    ClosePaymentFee?: XRPLNumber;
    OverpaymentFee?: number;
    InterestRate?: number;
    LateInterestRate?: number;
    CloseInterestRate?: number;
    OverpaymentInterestRate?: number;
    PaymentTotal?: number;
    PaymentInterval?: number;
    GracePeriod?: number;
    Flags?: number | LoanSetFlagsInterface;
}
export interface CounterpartySignature {
    SigningPubKey?: string;
    TxnSignature?: string;
    Signers?: Signer[];
}
export declare enum LoanSetFlags {
    tfLoanOverpayment = 65536
}
export interface LoanSetFlagsInterface extends GlobalFlagsInterface {
    tfLoanOverpayment?: boolean;
}
export declare function validateLoanSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanSet.d.ts.map