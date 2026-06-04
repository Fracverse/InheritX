import { Amount } from '../common';
import { Account, BaseTransaction, GlobalFlagsInterface } from './common';
import type { TransactionMetadataBase } from './metadata';
export declare enum NFTokenMintFlags {
    tfBurnable = 1,
    tfOnlyXRP = 2,
    tfTrustLine = 4,
    tfTransferable = 8,
    tfMutable = 16
}
export interface NFTokenMintFlagsInterface extends GlobalFlagsInterface {
    tfBurnable?: boolean;
    tfOnlyXRP?: boolean;
    tfTrustLine?: boolean;
    tfTransferable?: boolean;
    tfMutable?: boolean;
}
export interface NFTokenMint extends BaseTransaction {
    TransactionType: 'NFTokenMint';
    NFTokenTaxon: number;
    Issuer?: Account;
    TransferFee?: number;
    URI?: string | null;
    Amount?: Amount;
    Expiration?: number;
    Destination?: Account;
    Flags?: number | NFTokenMintFlagsInterface;
}
export interface NFTokenMintMetadata extends TransactionMetadataBase {
    nftoken_id?: string;
    offer_id?: string;
}
export declare function validateNFTokenMint(tx: Record<string, unknown>): void;
//# sourceMappingURL=NFTokenMint.d.ts.map