import { BaseTransaction, Account, GlobalFlagsInterface } from './common';
export declare enum MPTokenAuthorizeFlags {
    tfMPTUnauthorize = 1
}
export interface MPTokenAuthorizeFlagsInterface extends GlobalFlagsInterface {
    tfMPTUnauthorize?: boolean;
}
export interface MPTokenAuthorize extends BaseTransaction {
    TransactionType: 'MPTokenAuthorize';
    MPTokenIssuanceID: string;
    Holder?: Account;
    Flags?: number | MPTokenAuthorizeFlagsInterface;
}
export declare function validateMPTokenAuthorize(tx: Record<string, unknown>): void;
//# sourceMappingURL=MPTokenAuthorize.d.ts.map