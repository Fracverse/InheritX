import { Amount } from '../common';
import { BaseTransaction, GlobalFlagsInterface } from './common';
export declare enum OfferCreateFlags {
    tfPassive = 65536,
    tfImmediateOrCancel = 131072,
    tfFillOrKill = 262144,
    tfSell = 524288,
    tfHybrid = 1048576
}
export interface OfferCreateFlagsInterface extends GlobalFlagsInterface {
    tfPassive?: boolean;
    tfImmediateOrCancel?: boolean;
    tfFillOrKill?: boolean;
    tfSell?: boolean;
    tfHybrid?: boolean;
}
export interface OfferCreate extends BaseTransaction {
    TransactionType: 'OfferCreate';
    Flags?: number | OfferCreateFlagsInterface;
    Expiration?: number;
    OfferSequence?: number;
    TakerGets: Amount;
    TakerPays: Amount;
    DomainID?: string;
}
export declare function validateOfferCreate(tx: Record<string, unknown>): void;
//# sourceMappingURL=offerCreate.d.ts.map