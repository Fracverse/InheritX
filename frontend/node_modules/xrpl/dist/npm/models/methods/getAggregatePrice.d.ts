import { BaseRequest, BaseResponse } from './baseMethod';
export interface GetAggregatePriceRequest extends BaseRequest {
    command: 'get_aggregate_price';
    base_asset: string;
    quote_asset: string;
    oracles: Array<{
        account: string;
        oracle_document_id: string | number;
    }>;
    trim?: number;
    trim_threshold?: number;
}
export interface GetAggregatePriceResponse extends BaseResponse {
    result: {
        entire_set: {
            mean: string;
            size: number;
            standard_deviation: string;
        };
        trimmed_set?: {
            mean: string;
            size: number;
            standard_deviation: string;
        };
        median: string;
        time: number;
        ledger_current_index: number;
        validated: boolean;
    };
}
//# sourceMappingURL=getAggregatePrice.d.ts.map