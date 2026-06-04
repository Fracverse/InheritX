import { Account, XRPLNumber } from '../transactions/common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface LoanBroker extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'LoanBroker';
    Flags: number;
    Sequence: number;
    LoanSequence: number;
    OwnerNode: string;
    VaultNode: string;
    VaultID: string;
    Account: Account;
    Owner: Account;
    OwnerCount?: number;
    Data?: string;
    ManagementFeeRate?: number;
    DebtTotal?: XRPLNumber;
    DebtMaximum: XRPLNumber;
    CoverAvailable?: XRPLNumber;
    CoverRateMinimum?: number;
    CoverRateLiquidation?: number;
}
//# sourceMappingURL=LoanBroker.d.ts.map