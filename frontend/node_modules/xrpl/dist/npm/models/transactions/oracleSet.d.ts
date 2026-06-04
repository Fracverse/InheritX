import { PriceData } from '../common';
import { BaseTransaction } from './common';
export interface OracleSet extends BaseTransaction {
    TransactionType: 'OracleSet';
    OracleDocumentID: number;
    LastUpdateTime: number;
    PriceDataSeries: PriceData[];
    Provider?: string;
    URI?: string;
    AssetClass?: string;
}
export declare function validateOracleSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=oracleSet.d.ts.map