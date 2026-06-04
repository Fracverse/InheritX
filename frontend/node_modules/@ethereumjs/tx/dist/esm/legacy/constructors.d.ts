import { LegacyTx } from './tx.ts';
import type { TxOptions } from '../types.ts';
import type { TxData, TxValuesArray } from './tx.ts';
/**
 * Instantiate a transaction from a data dictionary.
 *
 * Format: { nonce, gasPrice, gasLimit, to, value, data, v, r, s }
 *
 * Notes:
 * - All parameters are optional and have some basic default values
 */
export declare function createLegacyTx(txData: TxData, opts?: TxOptions): LegacyTx;
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[nonce, gasPrice, gasLimit, to, value, data, v, r, s]`
 */
export declare function createLegacyTxFromBytesArray(values: TxValuesArray, opts?: TxOptions): LegacyTx;
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `rlp([nonce, gasPrice, gasLimit, to, value, data,
 * signatureV, signatureR, signatureS])`
 */
export declare function createLegacyTxFromRLP(serialized: Uint8Array, opts?: TxOptions): LegacyTx;
//# sourceMappingURL=constructors.d.ts.map