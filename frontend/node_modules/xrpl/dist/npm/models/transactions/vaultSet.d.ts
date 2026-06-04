import { BaseTransaction, XRPLNumber } from './common';
export interface VaultSet extends BaseTransaction {
    TransactionType: 'VaultSet';
    VaultID: string;
    Data?: string;
    AssetsMaximum?: XRPLNumber;
    DomainID?: string;
}
export declare function validateVaultSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=vaultSet.d.ts.map