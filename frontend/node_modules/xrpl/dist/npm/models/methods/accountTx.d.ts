import { APIVersion, DEFAULT_API_VERSION, RIPPLED_API_V1, RIPPLED_API_V2, ResponseOnlyTxInfo } from '../common';
import { Transaction, TransactionMetadata } from '../transactions';
import { BaseRequest, BaseResponse, LookupByLedgerRequest } from './baseMethod';
export interface AccountTxRequest extends BaseRequest, LookupByLedgerRequest {
    command: 'account_tx';
    account: string;
    ledger_index_min?: number;
    ledger_index_max?: number;
    binary?: boolean;
    forward?: boolean;
    limit?: number;
    marker?: unknown;
}
export interface AccountTxTransaction<Version extends APIVersion = typeof DEFAULT_API_VERSION> {
    ledger_index: number;
    meta: string | TransactionMetadata;
    tx_json?: Version extends typeof RIPPLED_API_V2 ? Transaction & ResponseOnlyTxInfo : never;
    tx?: Version extends typeof RIPPLED_API_V1 ? Transaction & ResponseOnlyTxInfo : never;
    hash?: Version extends typeof RIPPLED_API_V2 ? string : never;
    tx_blob?: string;
    validated: boolean;
}
interface AccountTxResponseBase<Version extends APIVersion = typeof DEFAULT_API_VERSION> extends BaseResponse {
    result: {
        account: string;
        ledger_index_min: number;
        ledger_index_max: number;
        limit: number;
        marker?: unknown;
        transactions: Array<AccountTxTransaction<Version>>;
        validated?: boolean;
    };
}
export type AccountTxResponse = AccountTxResponseBase;
export type AccountTxV1Response = AccountTxResponseBase<typeof RIPPLED_API_V1>;
export type AccountTxVersionResponseMap<Version extends APIVersion = typeof DEFAULT_API_VERSION> = Version extends typeof RIPPLED_API_V1 ? AccountTxV1Response : AccountTxResponse;
export {};
//# sourceMappingURL=accountTx.d.ts.map