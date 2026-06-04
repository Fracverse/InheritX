import { BaseTransaction, Transaction, TransactionMetadata } from '../transactions';
import { BaseRequest, BaseResponse } from './baseMethod';
export type SimulateRequest = BaseRequest & {
    command: 'simulate';
    binary?: boolean;
} & ({
    tx_blob: string;
    tx_json?: never;
} | {
    tx_json: Transaction;
    tx_blob?: never;
});
export type SimulateBinaryRequest = SimulateRequest & {
    binary: true;
};
export type SimulateJsonRequest = SimulateRequest & {
    binary?: false;
};
export type SimulateResponse = SimulateJsonResponse | SimulateBinaryResponse;
export interface SimulateBinaryResponse extends BaseResponse {
    result: {
        applied: false;
        engine_result: string;
        engine_result_code: number;
        engine_result_message: string;
        tx_blob: string;
        meta_blob: string;
        ledger_index: number;
    };
}
export interface SimulateJsonResponse<T extends BaseTransaction = Transaction> extends BaseResponse {
    result: {
        applied: false;
        engine_result: string;
        engine_result_code: number;
        engine_result_message: string;
        ledger_index: number;
        tx_json: T;
        meta?: TransactionMetadata<T>;
    };
}
//# sourceMappingURL=simulate.d.ts.map