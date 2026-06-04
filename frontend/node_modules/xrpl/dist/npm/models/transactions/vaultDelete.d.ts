import { BaseTransaction } from './common';
export interface VaultDelete extends BaseTransaction {
    TransactionType: 'VaultDelete';
    VaultID: string;
}
export declare function validateVaultDelete(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultDelete.d.ts.map