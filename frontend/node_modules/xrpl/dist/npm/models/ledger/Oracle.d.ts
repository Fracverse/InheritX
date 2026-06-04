import { PriceData } from '../common';
import { BaseLedgerEntry, HasPreviousTxnID } from './BaseLedgerEntry';
export default interface Oracle extends BaseLedgerEntry, HasPreviousTxnID {
    LedgerEntryType: 'Oracle';
    LastUpdateTime: number;
    Owner: string;
    AssetClass: string;
    Provider: string;
    PriceDataSeries: PriceData[];
    Flags: 0;
}
//# sourceMappingURL=Oracle.d.ts.map