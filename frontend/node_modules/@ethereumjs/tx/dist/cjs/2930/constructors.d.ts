import { AccessList2930Tx } from './tx.ts';
import type { TxOptions } from '../types.ts';
import type { TxData, TxValuesArray } from './tx.ts';
/**
 * Instantiate a transaction from a data dictionary.
 *
 * Format: { chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
 * v, r, s }
 *
 * Notes:
 * - `chainId` will be set automatically if not provided
 * - All parameters are optional and have some basic default values
 */
export declare function createAccessList2930Tx(txData: TxData, opts?: TxOptions): AccessList2930Tx;
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
 * signatureYParity (v), signatureR (r), signatureS (s)]`
 */
export declare function createAccessList2930TxFromBytesArray(values: TxValuesArray, opts?: TxOptions): AccessList2930Tx;
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `0x01 || rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
 * signatureYParity (v), signatureR (r), signatureS (s)])`
 */
export declare function createAccessList2930TxFromRLP(serialized: Uint8Array, opts?: TxOptions): AccessList2930Tx;
//# sourceMappingURL=constructors.d.ts.map