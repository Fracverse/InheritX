"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getHashedMessageToSign = getHashedMessageToSign;
exports.serialize = serialize;
exports.validateYParity = validateYParity;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const internal_ts_1 = require("../util/internal.js");
const legacy_ts_1 = require("./legacy.js");
/**
 * Gets the hashed message to sign for EIP-2718 transactions
 * @param tx - The EIP-2718 compatible transaction
 * @returns Hashed message to sign
 */
function getHashedMessageToSign(tx) {
    const keccakFunction = tx.common.customCrypto.keccak256 ?? sha3_js_1.keccak_256;
    return keccakFunction(tx.getMessageToSign());
}
/**
 * Serializes an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @param base - Optional base input for RLP encoding
 * @returns Serialized transaction bytes
 */
function serialize(tx, base) {
    return (0, util_1.concatBytes)((0, internal_ts_1.txTypeBytes)(tx.type), rlp_1.RLP.encode(base ?? tx.raw()));
}
/**
 * Validates the y-parity value of an EIP-2718 transaction
 * @param tx - The EIP-2718 compatible transaction
 * @throws EthereumJSErrorWithoutCode if y-parity is invalid
 */
function validateYParity(tx) {
    const { v } = tx;
    if (v !== undefined && v !== util_1.BIGINT_0 && v !== util_1.BIGINT_1) {
        const msg = (0, legacy_ts_1.errorMsg)(tx, 'The y-parity of the transaction should either be 0 or 1');
        throw (0, util_1.EthereumJSErrorWithoutCode)(msg);
    }
}
//# sourceMappingURL=eip2718.js.map