"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getDataGas = getDataGas;
exports.verifyAuthorizationList = verifyAuthorizationList;
const EIP2930 = require("./eip2930.js");
const util_1 = require("@ethereumjs/util");
/**
 * The amount of gas paid for the data in this tx
 */
function getDataGas(tx) {
    const eip2930Cost = EIP2930.getDataGas(tx);
    const eip7702Cost = BigInt(tx.authorizationList.length * Number(tx.common.param('perEmptyAccountCost')));
    return eip2930Cost + eip7702Cost;
}
/**
 * Checks if the authorization list is valid. Throws if invalid.
 * @param tx - Transaction whose authorization list should be validated
 */
function verifyAuthorizationList(tx) {
    const authorizationList = tx.authorizationList;
    if (authorizationList.length === 0) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: authorization list is empty');
    }
    for (const item of authorizationList) {
        if (item.length !== 6) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: authorization list item should have 6 elements');
        }
        for (const member of item) {
            // This checks if the `member` is a list, not bytes
            // This checks that the authority list does not have any embedded lists in it
            if (Array.isArray(member)) {
                throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: authority list element is a list, not bytes');
            }
        }
        const [chainId, address, nonce, yParity, r, s] = item;
        (0, util_1.validateNoLeadingZeroes)({ yParity, r, s, nonce, chainId });
        if (address.length !== 20) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: address length should be 20 bytes');
        }
        if ((0, util_1.bytesToBigInt)(chainId) > util_1.MAX_INTEGER) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: chainId exceeds 2^256 - 1');
        }
        if ((0, util_1.bytesToBigInt)(nonce) > util_1.MAX_UINT64) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: nonce exceeds 2^64 - 1');
        }
        const yParityBigInt = (0, util_1.bytesToBigInt)(yParity);
        if (yParityBigInt >= BigInt(2 ** 8)) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: yParity should be fit within 1 byte (0 - 255)');
        }
        if ((0, util_1.bytesToBigInt)(r) > util_1.MAX_INTEGER) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: r exceeds 2^256 - 1');
        }
        if ((0, util_1.bytesToBigInt)(s) > util_1.MAX_INTEGER) {
            throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction: s exceeds 2^256 - 1');
        }
    }
}
//# sourceMappingURL=eip7702.js.map