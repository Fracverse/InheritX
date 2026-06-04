import { LoanSet } from '../models';
import type { Wallet } from '.';
export declare function signLoanSetByCounterparty(wallet: Wallet, transaction: LoanSet | string, opts?: {
    multisign?: boolean | string;
}): {
    tx: LoanSet;
    tx_blob: string;
    hash: string;
};
export declare function combineLoanSetCounterpartySigners(transactions: Array<LoanSet | string>): {
    tx: LoanSet;
    tx_blob: string;
};
//# sourceMappingURL=counterpartySigner.d.ts.map