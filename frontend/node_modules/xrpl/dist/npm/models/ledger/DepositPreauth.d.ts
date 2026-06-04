import { AuthorizeCredential } from '../common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface DepositPreauth extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'DepositPreauth';
    Account: string;
    Flags: 0;
    OwnerNode: string;
    Authorize?: string;
    AuthorizeCredentials?: AuthorizeCredential[];
}
//# sourceMappingURL=DepositPreauth.d.ts.map