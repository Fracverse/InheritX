import { Amount, MPTAmount } from '../common';
import { BaseTransaction, Account } from './common';
export interface VaultWithdraw extends BaseTransaction {
    TransactionType: 'VaultWithdraw';
    VaultID: string;
    Amount: Amount | MPTAmount;
    Destination?: Account;
    DestinationTag?: number;
}
export declare function validateVaultWithdraw(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultWithdraw.d.ts.map