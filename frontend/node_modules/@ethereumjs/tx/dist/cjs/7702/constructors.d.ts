import { EOACode7702Tx } from './tx.ts';
import type { TxOptions } from '../types.ts';
import type { TxData, TxValuesArray } from './tx.ts';
/**
 * Instantiate a transaction from a data dictionary.
 *
 * Format: { chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, v, r, s }
 *
 * Notes:
 * - `chainId` will be set automatically if not provided
 * - All parameters are optional and have some basic default values
 */
export declare function createEOACode7702Tx(txData: TxData, opts?: TxOptions): EOACode7702Tx;
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, authorityList, signatureYParity, signatureR, signatureS]`
 */
export declare function createEOACode7702TxFromBytesArray(values: TxValuesArray, opts?: TxOptions): EOACode7702Tx;
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `0x04 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, signatureYParity, signatureR, signatureS])`
 */
export declare function createEOACode7702TxFromRLP(serialized: Uint8Array, opts?: TxOptions): EOACode7702Tx;
//# sourceMappingURL=constructors.d.ts.map