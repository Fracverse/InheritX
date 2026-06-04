import type { Input } from '@ethereumjs/rlp';
import type { EIP2718CompatibleTx } from '../types.ts';
/**
 * Gets the hashed message to sign for EIP-2718 transactions
 * @param tx - The EIP-2718 compatible transaction
 * @returns Hashed message to sign
 */
export declare function getHashedMessageToSign(tx: EIP2718CompatibleTx): Uint8Array;
/**
 * Serializes an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @param base - Optional base input for RLP encoding
 * @returns Serialized transaction bytes
 */
export declare function serialize(tx: EIP2718CompatibleTx, base?: Input): Uint8Array;
/**
 * Validates the y-parity value of an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @throws EthereumJSErrorWithoutCode if y-parity is invalid
 */
export declare function validateYParity(tx: EIP2718CompatibleTx): void;
//# sourceMappingURL=eip2718.d.ts.map