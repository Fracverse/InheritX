import { FeeMarket1559Tx } from './tx.ts';
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
export declare function createFeeMarket1559Tx(txData: TxData, opts?: TxOptions): FeeMarket1559Tx;
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, signatureYParity, signatureR, signatureS]`
 */
export declare function create1559FeeMarketTxFromBytesArray(values: TxValuesArray, opts?: TxOptions): FeeMarket1559Tx;
/**
 * Instantiate a transaction from an RLP serialized tx.
 *
 * Format: `0x02 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, signatureYParity, signatureR, signatureS])`
 */
export declare function createFeeMarket1559TxFromRLP(serialized: Uint8Array, opts?: TxOptions): FeeMarket1559Tx;
//# sourceMappingURL=constructors.d.ts.map