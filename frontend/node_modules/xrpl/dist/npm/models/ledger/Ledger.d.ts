import { APIVersion, DEFAULT_API_VERSION, RIPPLED_API_V1 } from '../common';
import { Transaction, TransactionMetadata } from '../transactions';
import { LedgerEntry } from './LedgerEntry';
interface BaseLedger {
    account_hash: string;
    accountState?: LedgerEntry[];
    close_flags: number;
    close_time: number;
    close_time_human: string;
    close_time_resolution: number;
    close_time_iso: string;
    closed: boolean;
    ledger_hash: string;
    parent_close_time: number;
    parent_hash: string;
    total_coins: string;
    transaction_hash: string;
    transactions?: Array<Transaction & {
        hash: string;
        metaData?: TransactionMetadata;
    }>;
}
export interface Ledger extends BaseLedger {
    ledger_index: number;
}
export interface LedgerV1 extends BaseLedger {
    ledger_index: string;
}
export type LedgerVersionMap<Version extends APIVersion = typeof DEFAULT_API_VERSION> = Version extends typeof RIPPLED_API_V1 ? LedgerV1 : Ledger;
export {};
//# sourceMappingURL=Ledger.d.ts.map