import { ClawbackAmount } from '../common';
import { BaseTransaction, Account } from './common';
export interface VaultClawback extends BaseTransaction {
    TransactionType: 'VaultClawback';
    VaultID: string;
    Holder: Account;
    Amount?: ClawbackAmount;
}
export declare function validateVaultClawback(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultClawback.d.ts.map