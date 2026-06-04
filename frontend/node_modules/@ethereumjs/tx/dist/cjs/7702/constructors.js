"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createEOACode7702Tx = createEOACode7702Tx;
exports.createEOACode7702TxFromBytesArray = createEOACode7702TxFromBytesArray;
exports.createEOACode7702TxFromRLP = createEOACode7702TxFromRLP;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const types_ts_1 = require("../types.js");
const internal_ts_1 = require("../util/internal.js");
const tx_ts_1 = require("./tx.js");
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
function createEOACode7702Tx(txData, opts = {}) {
    return new tx_ts_1.EOACode7702Tx(txData, opts);
}
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, authorityList, signatureYParity, signatureR, signatureS]`
 */
function createEOACode7702TxFromBytesArray(values, opts = {}) {
    if (values.length !== 10 && values.length !== 13) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid EIP-7702 transaction. Only expecting 10 values (for unsigned tx) or 13 values (for signed tx).');
    }
    const [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, authorityList, v, r, s,] = values;
    (0, internal_ts_1.validateNotArray)({ chainId, v });
    (0, util_1.validateNoLeadingZeroes)({ nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, value, v, r, s });
    return new tx_ts_1.EOACode7702Tx({
        chainId: (0, util_1.bytesToBigInt)(chainId),
        nonce,
        maxPriorityFeePerGas,
        maxFeePerGas,
        gasLimit,
        to,
        value,
        data,
        accessList: accessList ?? [],
        authorizationList: authorityList ?? [],
        v: v !== undefined ? (0, util_1.bytesToBigInt)(v) : undefined, // EIP2930 supports v's with value 0 (empty Uint8Array)
        r,
        s,
    }, opts);
}
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `0x04 || rlp([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data,
 * accessList, signatureYParity, signatureR, signatureS])`
 */
function createEOACode7702TxFromRLP(serialized, opts = {}) {
    if ((0, util_1.equalsBytes)(serialized.subarray(0, 1), (0, internal_ts_1.txTypeBytes)(types_ts_1.TransactionType.EOACodeEIP7702)) === false) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`Invalid serialized tx input: not an EIP-7702 transaction (wrong tx type, expected: ${types_ts_1.TransactionType.EOACodeEIP7702}, received: ${(0, util_1.bytesToHex)(serialized.subarray(0, 1))}`);
    }
    const values = rlp_1.RLP.decode(serialized.subarray(1));
    if (!Array.isArray(values)) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid serialized tx input: must be array');
    }
    return createEOACode7702TxFromBytesArray(values, opts);
}
//# sourceMappingURL=constructors.js.map