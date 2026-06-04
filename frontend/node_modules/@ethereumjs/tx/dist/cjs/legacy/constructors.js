"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createLegacyTx = createLegacyTx;
exports.createLegacyTxFromBytesArray = createLegacyTxFromBytesArray;
exports.createLegacyTxFromRLP = createLegacyTxFromRLP;
const rlp_1 = require("@ethereumjs/rlp");
const util_1 = require("@ethereumjs/util");
const tx_ts_1 = require("./tx.js");
/**
 * Instantiate a transaction from a data dictionary.
 *
 * Format: { nonce, gasPrice, gasLimit, to, value, data, v, r, s }
 *
 * Notes:
 * - All parameters are optional and have some basic default values
 */
function createLegacyTx(txData, opts = {}) {
    return new tx_ts_1.LegacyTx(txData, opts);
}
/**
 * Create a transaction from an array of byte encoded values ordered according to the devp2p network encoding - format noted below.
 *
 * Format: `[nonce, gasPrice, gasLimit, to, value, data, v, r, s]`
 */
function createLegacyTxFromBytesArray(values, opts = {}) {
    // If length is not 6, it has length 9. If v/r/s are empty Uint8Arrays, it is still an unsigned transaction
    // This happens if you get the RLP data from `raw()`
    if (values.length !== 6 && values.length !== 9) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid transaction. Only expecting 6 values (for unsigned tx) or 9 values (for signed tx).');
    }
    const [nonce, gasPrice, gasLimit, to, value, data, v, r, s] = values;
    (0, util_1.validateNoLeadingZeroes)({ nonce, gasPrice, gasLimit, value, v, r, s });
    return new tx_ts_1.LegacyTx({
        nonce,
        gasPrice,
        gasLimit,
        to,
        value,
        data,
        v,
        r,
        s,
    }, opts);
}
/**
 * Instantiate a transaction from a RLP serialized tx.
 *
 * Format: `rlp([nonce, gasPrice, gasLimit, to, value, data,
 * signatureV, signatureR, signatureS])`
 */
function createLegacyTxFromRLP(serialized, opts = {}) {
    const values = rlp_1.RLP.decode(serialized);
    if (!Array.isArray(values)) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Invalid serialized tx input. Must be array');
    }
    return createLegacyTxFromBytesArray(values, opts);
}
//# sourceMappingURL=constructors.js.map