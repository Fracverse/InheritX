import { Amount } from '../common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export interface Book {
    Book: {
        BookDirectory: string;
        BookNode: string;
    };
}
export default interface Offer extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Offer';
    Flags: number;
    Account: string;
    Sequence: number;
    TakerPays: Amount;
    TakerGets: Amount;
    BookDirectory: string;
    BookNode: string;
    OwnerNode: string;
    Expiration?: number;
    DomainID?: string;
    AdditionalBooks?: Book[];
}
export declare enum OfferFlags {
    lsfPassive = 65536,
    lsfSell = 131072,
    lsfHybrid = 262144
}
//# sourceMappingURL=Offer.d.ts.map