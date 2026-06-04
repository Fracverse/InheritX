import { APIVersion, DEFAULT_API_VERSION, RIPPLED_API_V1 } from '../common';
import { Ledger, LedgerV1, LedgerVersionMap } from '../ledger/Ledger';
import { LedgerEntryFilter } from '../ledger/LedgerEntry';
import { Transaction, TransactionAndMetadata } from '../transactions';
import { TransactionMetadata } from '../transactions/metadata';
import { BaseRequest, BaseResponse, LookupByLedgerRequest } from './baseMethod';
export interface LedgerRequest extends BaseRequest, LookupByLedgerRequest {
    command: 'ledger';
    full?: boolean;
    accounts?: boolean;
    transactions?: boolean;
    expand?: boolean;
    owner_funds?: boolean;
    binary?: boolean;
    queue?: boolean;
    type?: LedgerEntryFilter;
}
export interface LedgerRequestExpandedTransactionsOnly extends LedgerRequest {
    expand: true;
    transactions: true;
}
export interface LedgerRequestExpandedAccountsOnly extends LedgerRequest {
    expand: true;
    accounts: true;
}
export interface LedgerRequestExpandedAccountsAndTransactions extends LedgerRequest {
    expand: true;
    accounts: true;
    transactions: true;
}
export interface LedgerRequestExpandedTransactionsBinary extends LedgerRequest {
    expand: true;
    transactions: true;
    binary: true;
}
export interface LedgerModifiedOfferCreateTransaction {
    transaction: Transaction;
    metadata: TransactionMetadata & {
        owner_funds: string;
    };
}
export interface LedgerQueueData {
    account: string;
    tx: TransactionAndMetadata | LedgerModifiedOfferCreateTransaction | {
        tx_blob: string;
    };
    retries_remaining: number;
    preflight_result: string;
    last_result?: string;
    auth_change?: boolean;
    fee?: string;
    fee_level?: string;
    max_spend_drops?: string;
}
export interface LedgerBinary extends Omit<Ledger, 'transactions' | 'accountState'> {
    accountState?: string[];
    transactions?: string[];
}
export interface LedgerBinaryV1 extends Omit<LedgerV1, 'transactions' | 'accountState'> {
    accountState?: string[];
    transactions?: string[];
}
interface LedgerResponseBase {
    ledger_hash: string;
    ledger_index: number;
    queue_data?: Array<LedgerQueueData | string>;
    validated?: boolean;
}
interface LedgerResponseResult extends LedgerResponseBase {
    ledger: LedgerBinary;
}
interface LedgerV1ResponseResult extends LedgerResponseBase {
    ledger: LedgerBinaryV1;
}
export interface LedgerResponse extends BaseResponse {
    result: LedgerResponseResult;
}
export interface LedgerV1Response extends BaseResponse {
    result: LedgerV1ResponseResult;
}
export type LedgerVersionResponseMap<Version extends APIVersion = typeof DEFAULT_API_VERSION> = Version extends typeof RIPPLED_API_V1 ? LedgerV1Response : LedgerResponse;
interface LedgerResponseExpandedResult<Version extends APIVersion = typeof DEFAULT_API_VERSION> extends LedgerResponseBase {
    ledger: LedgerVersionMap<Version>;
}
export interface LedgerResponseExpanded<Version extends APIVersion = typeof DEFAULT_API_VERSION> extends BaseResponse {
    result: LedgerResponseExpandedResult<Version>;
}
export {};
//# sourceMappingURL=ledger.d.ts.map