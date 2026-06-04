import { Currency } from '../common';
import { Account } from '../transactions/common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface Vault extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Vault';
    LedgerIndex: string;
    Flags: number;
    Sequence: number;
    OwnerNode: string;
    Owner: string;
    Account: Account;
    Asset: Currency;
    AssetsTotal?: string;
    AssetsAvailable?: string;
    LossUnrealized?: string;
    ShareMPTID: string;
    WithdrawalPolicy: number;
    AssetsMaximum?: string;
    Data?: string;
    Scale?: number;
}
export declare enum VaultFlags {
    lsfVaultPrivate = 65536
}
//# sourceMappingURL=Vault.d.ts.map