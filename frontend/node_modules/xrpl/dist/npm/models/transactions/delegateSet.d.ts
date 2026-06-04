import { BaseTransaction, Account } from './common';
export interface Permission {
    Permission: {
        PermissionValue: string;
    };
}
export interface DelegateSet extends BaseTransaction {
    TransactionType: 'DelegateSet';
    Authorize: Account;
    Permissions: Permission[];
}
export declare function validateDelegateSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=delegateSet.d.ts.map