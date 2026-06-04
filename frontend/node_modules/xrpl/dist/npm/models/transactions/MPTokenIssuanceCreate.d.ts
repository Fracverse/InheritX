import { BaseTransaction, GlobalFlagsInterface } from './common';
import type { TransactionMetadataBase } from './metadata';
export declare enum MPTokenIssuanceCreateFlags {
    tfMPTCanLock = 2,
    tfMPTRequireAuth = 4,
    tfMPTCanEscrow = 8,
    tfMPTCanTrade = 16,
    tfMPTCanTransfer = 32,
    tfMPTCanClawback = 64
}
export interface MPTokenIssuanceCreateFlagsInterface extends GlobalFlagsInterface {
    tfMPTCanLock?: boolean;
    tfMPTRequireAuth?: boolean;
    tfMPTCanEscrow?: boolean;
    tfMPTCanTrade?: boolean;
    tfMPTCanTransfer?: boolean;
    tfMPTCanClawback?: boolean;
}
export interface MPTokenIssuanceCreate extends BaseTransaction {
    TransactionType: 'MPTokenIssuanceCreate';
    AssetScale?: number;
    MaximumAmount?: string;
    TransferFee?: number;
    MPTokenMetadata?: string;
    Flags?: number | MPTokenIssuanceCreateFlagsInterface;
}
export interface MPTokenIssuanceCreateMetadata extends TransactionMetadataBase {
    mpt_issuance_id?: string;
}
export declare function validateMPTokenIssuanceCreate(tx: Record<string, unknown>): void;
//# sourceMappingURL=MPTokenIssuanceCreate.d.ts.map