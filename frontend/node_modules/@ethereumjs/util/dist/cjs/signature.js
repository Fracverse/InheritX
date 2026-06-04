"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashPersonalMessage = exports.isValidSignature = exports.fromRPCSig = exports.toCompactSig = exports.toRPCSig = exports.ecrecover = void 0;
exports.calculateSigRecovery = calculateSigRecovery;
const secp256k1_js_1 = require("@noble/curves/secp256k1.js");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const bytes_ts_1 = require("./bytes.js");
const constants_ts_1 = require("./constants.js");
const errors_ts_1 = require("./errors.js");
const helpers_ts_1 = require("./helpers.js");
function calculateSigRecovery(v, chainId) {
    if (v === constants_ts_1.BIGINT_0 || v === constants_ts_1.BIGINT_1)
        return v;
    if (chainId === undefined) {
        return v - constants_ts_1.BIGINT_27;
    }
    return v - (chainId * constants_ts_1.BIGINT_2 + BigInt(35));
}
function isValidSigRecovery(recovery) {
    return recovery === constants_ts_1.BIGINT_0 || recovery === constants_ts_1.BIGINT_1;
}
/**
 * ECDSA public key recovery from signature.
 * NOTE: Accepts `v === 0 | v === 1` for EIP1559 transactions
 * @returns Recovered public key
 */
const ecrecover = function (msgHash, v, r, s, chainId) {
    const signature = (0, bytes_ts_1.concatBytes)((0, bytes_ts_1.setLengthLeft)(r, 32), (0, bytes_ts_1.setLengthLeft)(s, 32));
    const recovery = calculateSigRecovery(v, chainId);
    if (!isValidSigRecovery(recovery)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Invalid signature v value');
    }
    const sig = secp256k1_js_1.secp256k1.Signature.fromBytes(signature).addRecoveryBit(Number(recovery));
    const senderPubKey = sig.recoverPublicKey(msgHash);
    return senderPubKey.toBytes(false).slice(1);
};
exports.ecrecover = ecrecover;
/**
 * Convert signature parameters into the format of `eth_sign` RPC method.
 * NOTE: Accepts `v === 0 | v === 1` for EIP1559 transactions
 * @returns Signature
 */
const toRPCSig = function (v, r, s, chainId) {
    const recovery = calculateSigRecovery(v, chainId);
    if (!isValidSigRecovery(recovery)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Invalid signature v value');
    }
    // geth (and the RPC eth_sign method) uses the 65 byte format used by Bitcoin
    return (0, bytes_ts_1.bytesToHex)((0, bytes_ts_1.concatBytes)((0, bytes_ts_1.setLengthLeft)(r, 32), (0, bytes_ts_1.setLengthLeft)(s, 32), (0, bytes_ts_1.bigIntToBytes)(v)));
};
exports.toRPCSig = toRPCSig;
/**
 * Convert signature parameters into the format of Compact Signature Representation (EIP-2098).
 * NOTE: Accepts `v === 0 | v === 1` for EIP1559 transactions
 * @returns Signature
 */
const toCompactSig = function (v, r, s, chainId) {
    const recovery = calculateSigRecovery(v, chainId);
    if (!isValidSigRecovery(recovery)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Invalid signature v value');
    }
    const ss = Uint8Array.from([...s]);
    if ((v > BigInt(28) && v % constants_ts_1.BIGINT_2 === constants_ts_1.BIGINT_1) || v === constants_ts_1.BIGINT_1 || v === BigInt(28)) {
        ss[0] |= 0x80;
    }
    return (0, bytes_ts_1.bytesToHex)((0, bytes_ts_1.concatBytes)((0, bytes_ts_1.setLengthLeft)(r, 32), (0, bytes_ts_1.setLengthLeft)(ss, 32)));
};
exports.toCompactSig = toCompactSig;
/**
 * Convert signature format of the `eth_sign` RPC method to signature parameters
 *
 * NOTE: For an extracted `v` value < 27 (see Geth bug https://github.com/ethereum/go-ethereum/issues/2053)
 * `v + 27` is returned for the `v` value
 * NOTE: After EIP1559, `v` could be `0` or `1` but this function assumes
 * it's a signed message (EIP-191 or EIP-712) adding `27` at the end. Remove if needed.
 */
const fromRPCSig = function (sig) {
    const bytes = (0, bytes_ts_1.hexToBytes)(sig);
    let r;
    let s;
    let v;
    if (bytes.length >= 65) {
        r = bytes.subarray(0, 32);
        s = bytes.subarray(32, 64);
        v = (0, bytes_ts_1.bytesToBigInt)(bytes.subarray(64));
    }
    else if (bytes.length === 64) {
        // Compact Signature Representation (https://eips.ethereum.org/EIPS/eip-2098)
        r = bytes.subarray(0, 32);
        s = bytes.subarray(32, 64);
        v = BigInt((0, bytes_ts_1.bytesToInt)(bytes.subarray(32, 33)) >> 7);
        s[0] &= 0x7f;
    }
    else {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Invalid signature length');
    }
    // support both versions of `eth_sign` responses
    if (v < 27) {
        // TODO: verify this behavior, and verify in which context this method (`fromRPCSig`) is used
        v = v + constants_ts_1.BIGINT_27;
    }
    return {
        v,
        r,
        s,
    };
};
exports.fromRPCSig = fromRPCSig;
/**
 * Validate a ECDSA signature.
 * NOTE: Accepts `v === 0 | v === 1` for EIP1559 transactions
 * @param homesteadOrLater Indicates whether this is being used on either the homestead hardfork or a later one
 */
const isValidSignature = function (v, r, s, homesteadOrLater = true, chainId) {
    if (r.length !== 32 || s.length !== 32) {
        return false;
    }
    if (!isValidSigRecovery(calculateSigRecovery(v, chainId))) {
        return false;
    }
    const rBigInt = (0, bytes_ts_1.bytesToBigInt)(r);
    const sBigInt = (0, bytes_ts_1.bytesToBigInt)(s);
    if (rBigInt === constants_ts_1.BIGINT_0 ||
        rBigInt >= constants_ts_1.SECP256K1_ORDER ||
        sBigInt === constants_ts_1.BIGINT_0 ||
        sBigInt >= constants_ts_1.SECP256K1_ORDER) {
        return false;
    }
    if (homesteadOrLater && sBigInt >= constants_ts_1.SECP256K1_ORDER_DIV_2) {
        return false;
    }
    return true;
};
exports.isValidSignature = isValidSignature;
/**
 * Returns the keccak-256 hash of `message`, prefixed with the header used by the `eth_sign` RPC call.
 * The output of this function can be fed into `ecsign` to produce the same signature as the `eth_sign`
 * call for a given `message`, or fed to `ecrecover` along with a signature to recover the public key
 * used to produce the signature.
 */
const hashPersonalMessage = function (message) {
    (0, helpers_ts_1.assertIsBytes)(message);
    const prefix = (0, bytes_ts_1.utf8ToBytes)(`\u0019Ethereum Signed Message:\n${message.length}`);
    return (0, sha3_js_1.keccak_256)((0, bytes_ts_1.concatBytes)(prefix, message));
};
exports.hashPersonalMessage = hashPersonalMessage;
//# sourceMappingURL=signature.js.map