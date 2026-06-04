import { AuthorizeCredential } from '../common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface PermissionedDomain extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'PermissionedDomain';
    Owner: string;
    AcceptedCredentials: AuthorizeCredential[];
    Flags: 0;
    OwnerNode: string;
    Sequence: number;
}
//# sourceMappingURL=PermissionedDomain.d.ts.map