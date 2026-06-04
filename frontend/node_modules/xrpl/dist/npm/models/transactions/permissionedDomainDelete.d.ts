import { BaseTransaction } from './common';
export interface PermissionedDomainDelete extends BaseTransaction {
    TransactionType: 'PermissionedDomainDelete';
    DomainID: string;
}
export declare function validatePermissionedDomainDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=permissionedDomainDelete.d.ts.map