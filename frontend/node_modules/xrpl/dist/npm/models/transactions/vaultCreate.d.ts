import { Currency } from '../common';
import { BaseTransaction, GlobalFlagsInterface, XRPLNumber } from './common';
export declare enum VaultWithdrawalPolicy {
    vaultStrategyFirstComeFirstServe = 1
}
export declare enum VaultCreateFlags {
    tfVaultPrivate = 65536,
    tfVaultShareNonTransferable = 131072
}
export interface VaultCreateFlagsInterface extends GlobalFlagsInterface {
    tfVaultPrivate?: boolean;
    tfVaultShareNonTransferable?: boolean;
}
export interface VaultCreate extends BaseTransaction {
    TransactionType: 'VaultCreate';
    Asset: Currency;
    Data?: string;
    AssetsMaximum?: XRPLNumber;
    MPTokenMetadata?: string;
    WithdrawalPolicy?: number;
    DomainID?: string;
    Scale?: number;
}
export declare function validateVaultCreate(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultCreate.d.ts.map