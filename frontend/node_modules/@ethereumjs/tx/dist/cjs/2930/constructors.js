"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAccessList2930Tx = createAccessList2930Tx;
exports.createAccessList2930TxFromBytesArray = createAccessList2930TxFromBytesArray;
exports.createAccessList2930TxFromRLP = createAccessList2930TxFromRLP;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const types_ts_1 = require("../types.js");
const internal_ts_1 = require("../util/internal.js");
const tx_ts_1 = require("./tx.js");
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
function createAccessList2930Tx(txData, opts = {}) {
    return new tx_ts_1.AccessList2930Tx(txData, opts);
}
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
 * signatureYParity (v), signatureR (r), signatureS (s)]`
 */
function createAccessList2930TxFromBytesArray(values, opts = {}) {
    if (values.length !== 8 && values.length !== 11) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-2930 transaction. Only expecting 8 values (for unsigned tx) or 11 values (for signed tx).');
    }
    const [chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, v, r, s] = values;
    (0, internal_ts_1.validateNotArray)({ chainId, v });
    (0, util_1.validateNoLeadingZeroes)({ nonce, gasPrice, gasLimit, value, v, r, s });
    const emptyAccessList = [];
    return new tx_ts_1.AccessList2930Tx({
        chainId: (0, util_1.bytesToBigInt)(chainId),
        nonce,
        gasPrice,
        gasLimit,
        to,
        value,
        data,
        accessList: accessList ?? emptyAccessList,
        v: v !== undefined ? (0, util_1.bytesToBigInt)(v) : undefined, // EIP2930 supports v's with value 0 (empty Uint8Array)
        r,
        s,
    }, opts);
}
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `0x01 || rlp([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList,
 * signatureYParity (v), signatureR (r), signatureS (s)])`
 */
function createAccessList2930TxFromRLP(serialized, opts = {}) {
    if ((0, util_1.equalsBytes)(serialized.subarray(0, 1), (0, internal_ts_1.txTypeBytes)(types_ts_1.TransactionType.AccessListEIP2930)) === false) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid serialized tx input: not an EIP-2930 transaction (wrong tx type, expected: ${types_ts_1.TransactionType.AccessListEIP2930}, received: ${(0, util_1.bytesToHex)(serialized.subarray(0, 1))}`);
    }
    const values = rlp_1.RLP.decode(Uint8Array.from(serialized.subarray(1)));
    if (!Array.isArray(values)) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid serialized tx input: must be array');
    }
    return createAccessList2930TxFromBytesArray(values, opts);
}
//# sourceMappingURL=constructors.js.map