import { Common } from '@ethereumjs/common';
import type { TransactionInterface, TransactionType, TxData, TxOptions } from '../types.ts';
/**
 * Gets a Common instance, creating a new one if none provided
 * @param common - Optional Common instance
 * @returns Common instance (copied if provided, new Mainnet instance if not)
 */
export declare function getCommon(common?: Common): Common;
/**
 * Converts a transaction type to its byte representation
 * @param txType - The transaction type
 * @returns Uint8Array representation of the transaction type
 */
export declare function txTypeBytes(txType: TransactionType): Uint8Array;
/**
 * Validates that transaction data fields are not arrays
 * @param values - Object containing transaction data fields
 * @throws EthereumJSErrorWithoutCode if any transaction field is an array
 */
export declare function validateNotArray(values: {
    [key: string]: any;
}): void;
/**
 * Validates that an object with BigInt values cannot exceed the specified bit limit.
 * @param values Object containing string keys and BigInt values
 * @param bits Number of bits to check (64 or 256)
 * @param cannotEqual Pass true if the number also cannot equal one less than the maximum value
 */
export declare function valueOverflowCheck(values: {
    [key: string]: bigint | undefined;
}, bits?: number, cannotEqual?: boolean): void;
type Mutable<T> = {
    -readonly [P in keyof T]: T[P];
};
/**
 * Shared constructor logic for all transaction types
 * Note: Uses Mutable type to write to readonly properties. Only call this in transaction constructors.
 * @param tx - Mutable transaction interface to initialize
 * @param txData - Transaction data
 * @param opts - Transaction options
 */
export declare function sharedConstructor(tx: Mutable<TransactionInterface>, txData: TxData[TransactionType], opts?: TxOptions): void;
/**
 * Converts a transaction to its base JSON representation
 * @param tx - The transaction interface
 * @returns JSON object with base transaction fields
 */
export declare function getBaseJSON(tx: TransactionInterface): {
    type: `0x${string}`;
    nonce: `0x${string}`;
    gasLimit: `0x${string}`;
    to: `0x${string}` | undefined;
    value: `0x${string}`;
    data: `0x${string}`;
    v: `0x${string}` | undefined;
    r: `0x${string}` | undefined;
    s: `0x${string}` | undefined;
    chainId: `0x${string}`;
    yParity: `0x${string}` | undefined;
};
export {};
//# sourceMappingURL=internal.d.ts.map