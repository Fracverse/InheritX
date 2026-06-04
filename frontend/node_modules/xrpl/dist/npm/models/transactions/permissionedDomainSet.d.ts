import { AuthorizeCredential } from '../common';
import { BaseTransaction } from './common';
export interface PermissionedDomainSet extends BaseTransaction {
    TransactionType: 'PermissionedDomainSet';
    DomainID?: string;
    AcceptedCredentials: AuthorizeCredential[];
}
export declare function validatePermissionedDomainSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=permissionedDomainSet.d.ts.map