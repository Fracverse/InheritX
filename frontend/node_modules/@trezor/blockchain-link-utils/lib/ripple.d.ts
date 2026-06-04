import type { AccountTxTransaction, ServerInfoResponse } from 'xrpl';
import type { ServerInfo, Transaction } from '@trezor/blockchain-link-types';
export declare const transformServerInfo: (payload: ServerInfoResponse) => Omit<ServerInfo, "url">;
export declare const transformTransaction: (hash: string | undefined, tx_json: NonNullable<AccountTxTransaction["tx_json"]>, meta: AccountTxTransaction["meta"] | undefined, descriptor?: string) => Transaction;
//# sourceMappingURL=ripple.d.ts.map