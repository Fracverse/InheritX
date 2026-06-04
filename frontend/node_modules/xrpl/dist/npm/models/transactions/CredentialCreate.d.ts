import { BaseTransaction } from './common';
export interface CredentialCreate extends BaseTransaction {
    TransactionType: 'CredentialCreate';
    Account: string;
    Subject: string;
    CredentialType: string;
    Expiration?: number;
    URI?: string;
}
export declare function validateCredentialCreate(tx: Record<string, unknown>): void;
//# sourceMappingURL=CredentialCreate.d.ts.map