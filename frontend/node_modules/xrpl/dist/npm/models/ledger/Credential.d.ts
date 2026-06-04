import { GlobalFlagsInterface } from '../transactions/common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export interface CredentialFlags extends GlobalFlagsInterface {
    lsfAccepted?: boolean;
}
export default interface Credential extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Credential';
    Flags: number | CredentialFlags;
    Subject: string;
    Issuer: string;
    CredentialType: string;
    SubjectNode: string;
    IssuerNode: string;
    Expiration?: number;
    URI?: string;
}
//# sourceMappingURL=Credential.d.ts.map