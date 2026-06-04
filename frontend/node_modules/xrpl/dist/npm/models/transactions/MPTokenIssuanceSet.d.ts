import { BaseTransaction, Account, GlobalFlagsInterface } from './common';
export declare enum MPTokenIssuanceSetFlags {
    tfMPTLock = 1,
    tfMPTUnlock = 2
}
export interface MPTokenIssuanceSetFlagsInterface extends GlobalFlagsInterface {
    tfMPTLock?: boolean;
    tfMPTUnlock?: boolean;
}
export interface MPTokenIssuanceSet extends BaseTransaction {
    TransactionType: 'MPTokenIssuanceSet';
    MPTokenIssuanceID: string;
    Holder?: Account;
    Flags?: number | MPTokenIssuanceSetFlagsInterface;
}
export declare function validateMPTokenIssuanceSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=MPTokenIssuanceSet.d.ts.map