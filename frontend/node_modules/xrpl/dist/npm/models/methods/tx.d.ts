import { APIVersion, DEFAULT_API_VERSION, RIPPLED_API_V1, RIPPLED_API_V2 } from '../common';
import { Transaction, TransactionMetadata } from '../transactions';
import { BaseTransaction } from '../transactions/common';
import { BaseRequest, BaseResponse } from './baseMethod';
export interface TxRequest extends BaseRequest {
    command: 'tx';
    transaction?: string;
    ctid?: string;
    binary?: boolean;
    min_ledger?: number;
    max_ledger?: number;
}
interface BaseTxResult<Version extends APIVersion = typeof DEFAULT_API_VERSION, T extends BaseTransaction = Transaction> {
    hash: string;
    ctid?: string;
    ledger_index?: number;
    meta_blob?: Version extends typeof RIPPLED_API_V2 ? TransactionMetadata<T> | string : never;
    meta?: TransactionMetadata<T> | string;
    validated?: boolean;
    close_time_iso?: string;
    date?: number;
}
export interface TxResponse<T extends BaseTransaction = Transaction> extends BaseResponse {
    result: BaseTxResult<typeof RIPPLED_API_V2, T> & {
        tx_json: T;
    };
    searched_all?: boolean;
}
export interface TxV1Response<T extends BaseTransaction = Transaction> extends BaseResponse {
    result: BaseTxResult<typeof RIPPLED_API_V1, T> & T;
    searched_all?: boolean;
}
export type TxVersionResponseMap<Version extends APIVersion = typeof DEFAULT_API_VERSION> = Version extends typeof RIPPLED_API_V1 ? TxV1Response : TxResponse;
export {};
//# sourceMappingURL=tx.d.ts.map