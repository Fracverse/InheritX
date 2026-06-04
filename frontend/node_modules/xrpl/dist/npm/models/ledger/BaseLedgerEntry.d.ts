export interface BaseLedgerEntry {
    index: string;
}
export interface HasPreviousTxnID {
    PreviousTxnID: string;
    PreviousTxnLgrSeq: number;
}
export interface HasOptionalPreviousTxnID {
    PreviousTxnID?: string;
    PreviousTxnLgrSeq?: number;
}
//# sourceMappingURL=BaseLedgerEntry.d.ts.map