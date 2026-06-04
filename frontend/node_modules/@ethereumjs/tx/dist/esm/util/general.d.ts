import type { TypedTxData } from '../types.ts';
/**
 * Normalizes values for transactions that are received from an RPC provider to be properly usable within
 * the ethereumjs context
 * @param txParamsFromRPC a transaction in the standard JSON-RPC format
 * @returns a normalized {@link TypedTxData} object with valid values
 */
export declare const normalizeTxParams: (txParamsFromRPC: any) => TypedTxData;
//# sourceMappingURL=general.d.ts.map