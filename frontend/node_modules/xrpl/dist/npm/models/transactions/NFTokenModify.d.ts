import { BaseTransaction, Account } from './common';
export interface NFTokenModify extends BaseTransaction {
    TransactionType: 'NFTokenModify';
    NFTokenID: string;
    Owner?: Account;
    URI?: string | null;
}
export declare function validateNFTokenModify(tx: Record<string, unknown>): void;
//# sourceMappingURL=NFTokenModify.d.ts.map