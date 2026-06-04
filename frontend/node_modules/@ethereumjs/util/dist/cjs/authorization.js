"use strict";
// Utility helpers to convert authorization lists from the byte format and JSON format and vice versa
Object.defineProperty(exports, "__esModule", { value: true });
exports.EOA_CODE_7702_AUTHORITY_SIGNING_MAGIC = void 0;
exports.eoaCode7702AuthorizationListBytesItemToJSON = eoaCode7702AuthorizationListBytesItemToJSON;
exports.eoaCode7702AuthorizationListJSONItemToBytes = eoaCode7702AuthorizationListJSONItemToBytes;
exports.eoaCode7702AuthorizationMessageToSign = eoaCode7702AuthorizationMessageToSign;
exports.eoaCode7702AuthorizationHashedMessageToSign = eoaCode7702AuthorizationHashedMessageToSign;
exports.eoaCode7702SignAuthorization = eoaCode7702SignAuthorization;
exports.eoaCode7702RecoverAuthority = eoaCode7702RecoverAuthority;
const rlp_1 = require("@ethereumjs/rlp");
const secp256k1_js_1 = require("@noble/curves/secp256k1.js");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const account_ts_1 = require("./account.js");
const address_ts_1 = require("./address.js");
const bytes_ts_1 = require("./bytes.js");
const signature_ts_1 = require("./signature.js");
exports.EOA_CODE_7702_AUTHORITY_SIGNING_MAGIC = (0, bytes_ts_1.hexToBytes)('0x05');
/**
 * Converts an authorization list to a JSON format
 * @param authorizationList
 * @returns authorizationList in JSON format
 */
function eoaCode7702AuthorizationListBytesItemToJSON(authorizationList) {
    const [chainId, address, nonce, yParity, r, s] = authorizationList;
    return {
        chainId: (0, bytes_ts_1.bytesToHex)(chainId),
        address: (0, bytes_ts_1.bytesToHex)(address),
        nonce: (0, bytes_ts_1.bytesToHex)(nonce),
        yParity: (0, bytes_ts_1.bytesToHex)(yParity),
        r: (0, bytes_ts_1.bytesToHex)(r),
        s: (0, bytes_ts_1.bytesToHex)(s),
    };
}
/**
 * Converts an authority list in JSON to a bytes format
 * @param authorizationList
 * @returns bytes format of the authority list
 */
function eoaCode7702AuthorizationListJSONItemToBytes(authorizationList) {
    const requiredFields = ['chainId', 'address', 'nonce', 'yParity', 'r', 's'];
    // Validate all required fields are present
    for (const field of requiredFields) {
        if (authorizationList[field] === undefined) {
            throw (0, rlp_1.EthereumJSErrorWithoutCode)(`EIP-7702 authorization list invalid: ${field} is not defined`);
        }
    }
    return [
        (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(authorizationList.chainId)),
        (0, bytes_ts_1.hexToBytes)(authorizationList.address),
        (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(authorizationList.nonce)),
        (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(authorizationList.yParity)),
        (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(authorizationList.r)),
        (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(authorizationList.s)),
    ];
}
/** Authorization signing utility methods */
function unsignedAuthorizationListToBytes(input) {
    const { chainId: chainIdHex, address: addressHex, nonce: nonceHex } = input;
    const chainId = (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(chainIdHex));
    const address = (0, bytes_ts_1.setLengthLeft)((0, bytes_ts_1.hexToBytes)(addressHex), 20);
    const nonce = (0, bytes_ts_1.unpadBytes)((0, bytes_ts_1.hexToBytes)(nonceHex));
    return [chainId, address, nonce];
}
/**
 * Returns the bytes (RLP-encoded) to sign
 * @param input Either the bytes or the object format of the authorization list item
 * @returns
 */
function eoaCode7702AuthorizationMessageToSign(input) {
    if (Array.isArray(input)) {
        // The address is validated, the chainId and nonce will be `unpadBytes` such that these are valid
        const [chainId, address, nonce] = input;
        if (address.length !== 20) {
            throw (0, rlp_1.EthereumJSErrorWithoutCode)('Cannot sign authority: address length should be 20 bytes');
        }
        return (0, bytes_ts_1.concatBytes)(exports.EOA_CODE_7702_AUTHORITY_SIGNING_MAGIC, rlp_1.RLP.encode([(0, bytes_ts_1.unpadBytes)(chainId), address, (0, bytes_ts_1.unpadBytes)(nonce)]));
    }
    else {
        const [chainId, address, nonce] = unsignedAuthorizationListToBytes(input);
        return (0, bytes_ts_1.concatBytes)(exports.EOA_CODE_7702_AUTHORITY_SIGNING_MAGIC, rlp_1.RLP.encode([chainId, address, nonce]));
    }
}
/**
 * Hashes the RLP-encoded message to sign
 * @param input
 * @returns
 */
function eoaCode7702AuthorizationHashedMessageToSign(input) {
    return (0, sha3_js_1.keccak_256)(eoaCode7702AuthorizationMessageToSign(input));
}
/**
 * Signs an authorization list item and returns it in `bytes` format.
 * To get the JSON format, use `authorizationListBytesToJSON([signed])[0] to convert it`
 * @param input
 * @param privateKey
 * @param ecSign
 * @returns
 */
function eoaCode7702SignAuthorization(input, privateKey, ecSign) {
    const msgHash = eoaCode7702AuthorizationHashedMessageToSign(input);
    const secp256k1Sign = (msgHash, pk) => {
        return (ecSign?.(msgHash, pk) ??
            secp256k1_js_1.secp256k1.Signature.fromBytes(secp256k1_js_1.secp256k1.sign(msgHash, pk, { prehash: false, format: 'recovered' }), 'recovered'));
    };
    const signed = secp256k1Sign(msgHash, privateKey);
    const [chainId, address, nonce] = Array.isArray(input)
        ? input
        : unsignedAuthorizationListToBytes(input);
    return [
        (0, bytes_ts_1.unpadBytes)(chainId),
        address,
        (0, bytes_ts_1.unpadBytes)(nonce),
        (0, bytes_ts_1.bigIntToUnpaddedBytes)(BigInt(signed.recovery)),
        (0, bytes_ts_1.bigIntToUnpaddedBytes)(signed.r),
        (0, bytes_ts_1.bigIntToUnpaddedBytes)(signed.s),
    ];
}
function eoaCode7702RecoverAuthority(input) {
    const inputBytes = Array.isArray(input)
        ? input
        : eoaCode7702AuthorizationListJSONItemToBytes(input);
    const [chainId, address, nonce, yParity, r, s] = [
        (0, bytes_ts_1.unpadBytes)(inputBytes[0]),
        inputBytes[1],
        (0, bytes_ts_1.unpadBytes)(inputBytes[2]),
        (0, bytes_ts_1.unpadBytes)(inputBytes[3]),
        (0, bytes_ts_1.unpadBytes)(inputBytes[4]),
        (0, bytes_ts_1.unpadBytes)(inputBytes[5]),
    ];
    const msgHash = eoaCode7702AuthorizationHashedMessageToSign([chainId, address, nonce]);
    const pubKey = (0, signature_ts_1.ecrecover)(msgHash, (0, bytes_ts_1.bytesToBigInt)(yParity), r, s);
    return new address_ts_1.Address((0, account_ts_1.publicToAddress)(pubKey));
}
//# sourceMappingURL=authorization.js.map