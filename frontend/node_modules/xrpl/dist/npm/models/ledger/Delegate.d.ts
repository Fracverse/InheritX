import { Permission } from '../transactions';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface Delegate extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Delegate';
    Account: string;
    Authorize: string;
    Permissions: Permission[];
    OwnerNode: string;
    Flags: 0;
}
//# sourceMappingURL=Delegate.d.ts.map