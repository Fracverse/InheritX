import { Amount, MPTAmount } from '../common';
import { BaseTransaction } from './common';
export interface VaultDeposit extends BaseTransaction {
    TransactionType: 'VaultDeposit';
    VaultID: string;
    Amount: Amount | MPTAmount;
}
export declare function validateVaultDeposit(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultDeposit.d.ts.map