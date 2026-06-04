import { Address } from '@ethereumjs/util';
import { TransactionType } from '../types.ts';
import type { LegacyTxInterface, Transaction } from '../types.ts';
/**
 * Creates an error message with transaction context
 * @param tx - The transaction interface
 * @param msg - The error message
 * @returns Formatted error message with transaction context
 */
export declare function errorMsg(tx: LegacyTxInterface, msg: string): string;
/**
 * Checks if a transaction is signed
 * @param tx - The transaction interface
 * @returns true if the transaction is signed
 */
export declare function isSigned(tx: LegacyTxInterface): boolean;
/**
 * The amount of gas paid for the data in this tx
 */
export declare function getDataGas(tx: LegacyTxInterface): bigint;
/**
 * The minimum gas limit which the tx to have to be valid.
 * This covers costs as the standard fee (21000 gas), the data fee (paid for each calldata byte),
 * the optional creation fee (if the transaction creates a contract), and if relevant the gas
 * to be paid for access lists (EIP-2930) and authority lists (EIP-7702).
 */
export declare function getIntrinsicGas(tx: LegacyTxInterface): bigint;
/**
 * Checks if the transaction targets the creation address (deploys a contract).
 * @param tx - Transaction interface to inspect
 * @returns true if the transaction's `to` is undefined or empty
 */
export declare function toCreationAddress(tx: LegacyTxInterface): boolean;
/**
 * Computes the keccak_256 hash of a signed legacy transaction.
 * @param tx - Transaction to hash
 * @returns Hash of the serialized transaction
 * @throws EthereumJSErrorWithoutCode if the transaction is unsigned
 */
export declare function hash(tx: LegacyTxInterface): Uint8Array;
/**
 * EIP-2: All transaction signatures whose s-value is greater than secp256k1n/2are considered invalid.
 * Reasoning: https://ethereum.stackexchange.com/a/55728
 */
export declare function validateHighS(tx: LegacyTxInterface): void;
/**
 * Recovers the sender's public key from the transaction signature.
 * @param tx - Transaction from which the public key should be derived
 * @returns The uncompressed sender public key
 * @throws EthereumJSErrorWithoutCode if the signature is invalid
 */
export declare function getSenderPublicKey(tx: LegacyTxInterface): Uint8Array;
/**
 * Calculates the effective priority fee for a legacy-style transaction.
 * @param gasPrice - Gas price specified on the transaction
 * @param baseFee - Optional base fee from the block when operating on L2s that mimic 1559 behavior
 * @returns The priority fee portion that can be paid to the block producer
 * @throws EthereumJSErrorWithoutCode if the base fee exceeds the gas price
 */
export declare function getEffectivePriorityFee(gasPrice: bigint, baseFee: bigint | undefined): bigint;
/**
 * Validates the transaction signature and minimum gas requirements.
 * @returns {string[]} an array of error strings
 */
export declare function getValidationErrors(tx: LegacyTxInterface): string[];
/**
 * Validates the transaction signature and minimum gas requirements.
 * @returns {boolean} true if the transaction is valid, false otherwise
 */
export declare function isValid(tx: LegacyTxInterface): boolean;
/**
 * Determines if the signature is valid
 */
export declare function verifySignature(tx: LegacyTxInterface): boolean;
/**
 * Returns the sender's address
 */
export declare function getSenderAddress(tx: LegacyTxInterface): Address;
/**
 * Signs a transaction.
 *
 * Note that the signed tx is returned as a new object,
 * use as follows:
 * ```javascript
 * const signedTx = tx.sign(privateKey)
 * ```
 */
export declare function sign(tx: LegacyTxInterface, privateKey: Uint8Array, extraEntropy?: Uint8Array | boolean): Transaction[TransactionType];
/**
 * Builds a compact string that summarizes common transaction fields for error messages.
 * @param tx - Transaction used to assemble the postfix
 * @returns A formatted string containing tx type, hash, nonce, value, signature status, and hardfork
 */
export declare function getSharedErrorPostfix(tx: LegacyTxInterface): string;
//# sourceMappingURL=legacy.d.ts.map