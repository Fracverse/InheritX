import { Amount, MPTAmount } from '../common';
import { BaseTransaction } from './common';
import { MPTokenIssuanceCreate, MPTokenIssuanceCreateMetadata } from './MPTokenIssuanceCreate';
import { NFTokenAcceptOffer, NFTokenAcceptOfferMetadata } from './NFTokenAcceptOffer';
import { NFTokenCancelOffer, NFTokenCancelOfferMetadata } from './NFTokenCancelOffer';
import { NFTokenCreateOffer, NFTokenCreateOfferMetadata } from './NFTokenCreateOffer';
import { NFTokenMint, NFTokenMintMetadata } from './NFTokenMint';
import { Payment, PaymentMetadata } from './payment';
import type { Transaction } from './transaction';
export interface CreatedNode {
    CreatedNode: {
        LedgerEntryType: string;
        LedgerIndex: string;
        NewFields: {
            [field: string]: unknown;
        };
    };
}
export interface ModifiedNode {
    ModifiedNode: {
        LedgerEntryType: string;
        LedgerIndex: string;
        FinalFields?: {
            [field: string]: unknown;
        };
        PreviousFields?: {
            [field: string]: unknown;
        };
        PreviousTxnID?: string;
        PreviousTxnLgrSeq?: number;
    };
}
export interface DeletedNode {
    DeletedNode: {
        LedgerEntryType: string;
        LedgerIndex: string;
        PreviousFields?: {
            [field: string]: unknown;
        };
        FinalFields: {
            [field: string]: unknown;
        };
    };
}
export type Node = CreatedNode | ModifiedNode | DeletedNode;
export declare function isCreatedNode(node: Node): node is CreatedNode;
export declare function isModifiedNode(node: Node): node is ModifiedNode;
export declare function isDeletedNode(node: Node): node is DeletedNode;
export interface TransactionMetadataBase {
    AffectedNodes: Node[];
    DeliveredAmount?: Amount | MPTAmount;
    delivered_amount?: Amount | MPTAmount | 'unavailable';
    TransactionIndex: number;
    TransactionResult: string;
    ParentBatchID?: string;
}
export type TransactionMetadata<T extends BaseTransaction = Transaction> = T extends Payment ? PaymentMetadata : T extends NFTokenMint ? NFTokenMintMetadata : T extends NFTokenCreateOffer ? NFTokenCreateOfferMetadata : T extends NFTokenAcceptOffer ? NFTokenAcceptOfferMetadata : T extends NFTokenCancelOffer ? NFTokenCancelOfferMetadata : T extends MPTokenIssuanceCreate ? MPTokenIssuanceCreateMetadata : TransactionMetadataBase;
//# sourceMappingURL=metadata.d.ts.map