import { BaseTransaction, GlobalFlagsInterface } from './common';
export interface LoanManage extends BaseTransaction {
    TransactionType: 'LoanManage';
    LoanID: string;
    Flags?: number | LoanManageFlagsInterface;
}
export declare enum LoanManageFlags {
    tfLoanDefault = 65536,
    tfLoanImpair = 131072,
    tfLoanUnimpair = 262144
}
export interface LoanManageFlagsInterface extends GlobalFlagsInterface {
    tfLoanDefault?: boolean;
    tfLoanImpair?: boolean;
    tfLoanUnimpair?: boolean;
}
export declare function validateLoanManage(tx: Record<string, unknown>): void;
//# sourceMappingURL=loanManage.d.ts.map