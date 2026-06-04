import { BaseTransaction } from './common';
export interface CredentialAccept extends BaseTransaction {
    TransactionType: 'CredentialAccept';
    Account: string;
    Issuer: string;
    CredentialType: string;
}
export declare function validateCredentialAccept(tx: Record<string, unknown>): void;
//# sourceMappingURL=CredentialAccept.d.ts.map