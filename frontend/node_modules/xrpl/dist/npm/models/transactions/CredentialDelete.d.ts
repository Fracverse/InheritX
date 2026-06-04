import { BaseTransaction } from './common';
export interface CredentialDelete extends BaseTransaction {
    TransactionType: 'CredentialDelete';
    Account: string;
    CredentialType: string;
    Subject?: string;
    Issuer?: string;
}
export declare function validateCredentialDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=CredentialDelete.d.ts.map