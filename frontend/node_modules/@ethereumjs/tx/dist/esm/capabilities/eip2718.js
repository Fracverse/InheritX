import { RLP } from '@ethereumjs/rlp';
import { BIGINT_0, BIGINT_1, EthereumJSErrorWithoutCode, concatBytes } from '@ethereumjs/util';
import { keccak_256 } from '@noble/hashes/sha3.js';
import { txTypeBytes } from "../util/internal.js";
import { errorMsg } from "./legacy.js";
/**
 * Gets the hashed message to sign for EIP-2718 transactions
 * @param tx - The EIP-2718 compatible transaction
 * @returns Hashed message to sign
 */
export function getHashedMessageToSign(tx) {
    const keccakFunction = tx.common.customCrypto.keccak256 ?? keccak_256;
    return keccakFunction(tx.getMessageToSign());
}
/**
 * Serializes an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @param base - Optional base input for RLP encoding
 * @returns Serialized transaction bytes
 */
export function serialize(tx, base) {
    return concatBytes(txTypeBytes(tx.type), RLP.encode(base ?? tx.raw()));
}
/**
 * Validates the y-parity value of an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @throws EthereumJSErrorWithoutCode if y-parity is invalid
 */
export function validateYParity(tx) {
    const { v } = tx;
    if (v !== undefined && v !== BIGINT_0 && v !== BIGINT_1) {
        const msg = errorMsg(tx, 'The y-parity of the transaction should either be 0 or 1');
        throw EthereumJSErrorWithoutCode(msg);
    }
}
//# sourceMappingURL=eip2718.js.map