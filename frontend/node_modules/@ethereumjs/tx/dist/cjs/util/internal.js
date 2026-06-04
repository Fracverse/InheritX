"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getCommon = getCommon;
exports.txTypeBytes = txTypeBytes;
exports.validateNotArray = validateNotArray;
exports.valueOverflowCheck = valueOverflowCheck;
exports.sharedConstructor = sharedConstructor;
exports.getBaseJSON = getBaseJSON;
const common_1 = require("@ethereumjs/common");
const util_1 = require("@ethereumjs/util");
const params_ts_1 = require("../params.js");
/**
 * Gets a Common instance, creating a new one if none provided
 * @param common - Optional Common instance
 * @returns Common instance (copied if provided, new Mainnet instance if not)
 */
function getCommon(common) {
    return common?.copy() ?? new common_1.Common({ chain: common_1.Mainnet });
}
/**
 * Converts a transaction type to its byte representation
 * @param txType - The transaction type
 * @returns Uint8Array representation of the transaction type
 */
function txTypeBytes(txType) {
    return (0, util_1.hexToBytes)(`0x${txType.toString(16).padStart(2, '0')}`);
}
/**
 * Validates that transaction data fields are not arrays
 * @param values - Object containing transaction data fields
 * @throws EthereumJSErrorWithoutCode if any transaction field is an array
 */
function validateNotArray(values) {
    const txDataKeys = [
        'nonce',
        'gasPrice',
        'gasLimit',
        'to',
        'value',
        'data',
        'v',
        'r',
        's',
        'type',
        'baseFee',
        'maxFeePerGas',
        'chainId',
    ];
    for (const [key, value] of Object.entries(values)) {
        if (txDataKeys.includes(key)) {
            if (Array.isArray(value)) {
                throw (0, util_1.EthereumJSErrorWithoutCode)(`${key} cannot be an array`);
            }
        }
    }
}
function checkMaxInitCodeSize(common, length) {
    const maxInitCodeSize = common.param('maxInitCodeSize');
    if (maxInitCodeSize && BigInt(length) > maxInitCodeSize) {
        throw (0, util_1.EthereumJSErrorWithoutCode)(`the initcode size of this transaction is too large: it is ${length} while the max is ${common.param('maxInitCodeSize')}`);
    }
}
/**
 * Validates that an object with BigInt values cannot exceed the specified bit limit.
 * @param values Object containing string keys and BigInt values
 * @param bits Number of bits to check (64 or 256)
 * @param cannotEqual Pass true if the number also cannot equal one less than the maximum value
 */
function valueOverflowCheck(values, bits = 256, cannotEqual = false) {
    for (const [key, value] of Object.entries(values)) {
        switch (bits) {
            case 64:
                if (cannotEqual) {
                    if (value !== undefined && value >= util_1.MAX_UINT64) {
                        // TODO: error msgs got raised to a error string handler first, now throws "generic" error
                        throw (0, util_1.EthereumJSErrorWithoutCode)(`${key} cannot equal or exceed MAX_UINT64 (2^64-1), given ${value}`);
                    }
                }
                else {
                    if (value !== undefined && value > util_1.MAX_UINT64) {
                        throw (0, util_1.EthereumJSErrorWithoutCode)(`${key} cannot exceed MAX_UINT64 (2^64-1), given ${value}`);
                    }
                }
                break;
            case 256:
                if (cannotEqual) {
                    if (value !== undefined && value >= util_1.MAX_INTEGER) {
                        throw (0, util_1.EthereumJSErrorWithoutCode)(`${key} cannot equal or exceed MAX_INTEGER (2^256-1), given ${value}`);
                    }
                }
                else {
                    if (value !== undefined && value > util_1.MAX_INTEGER) {
                        throw (0, util_1.EthereumJSErrorWithoutCode)(`${key} cannot exceed MAX_INTEGER (2^256-1), given ${value}`);
                    }
                }
                break;
            default: {
                throw (0, util_1.EthereumJSErrorWithoutCode)('unimplemented bits value');
            }
        }
    }
}
/**
 * Shared constructor logic for all transaction types
 * Note: Uses Mutable type to write to readonly properties. Only call this in transaction constructors.
 * @param tx - Mutable transaction interface to initialize
 * @param txData - Transaction data
 * @param opts - Transaction options
 */
function sharedConstructor(tx, txData, opts = {}) {
    // LOAD base tx super({ ...txData, type: TransactionType.Legacy }, opts)
    tx.common = getCommon(opts.common);
    tx.common.updateParams(opts.params ?? params_ts_1.paramsTx);
    validateNotArray(txData); // is this necessary?
    const { nonce, gasLimit, to, value, data, v, r, s } = txData;
    tx.txOptions = opts; // TODO: freeze?
    // Set the tx properties
    const toB = (0, util_1.toBytes)(to === '' ? '0x' : to);
    tx.to = toB.length > 0 ? new util_1.Address(toB) : undefined; // TODO mark this explicitly as null if create-contract-tx?
    const vB = (0, util_1.toBytes)(v);
    const rB = (0, util_1.toBytes)(r);
    const sB = (0, util_1.toBytes)(s);
    tx.nonce = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(nonce));
    tx.gasLimit = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(gasLimit));
    tx.to = toB.length > 0 ? new util_1.Address(toB) : undefined;
    tx.value = (0, util_1.bytesToBigInt)((0, util_1.toBytes)(value));
    tx.data = (0, util_1.toBytes)(data === '' ? '0x' : data);
    // Set signature values (if the tx is signed)
    tx.v = vB.length > 0 ? (0, util_1.bytesToBigInt)(vB) : undefined;
    tx.r = rB.length > 0 ? (0, util_1.bytesToBigInt)(rB) : undefined;
    tx.s = sB.length > 0 ? (0, util_1.bytesToBigInt)(sB) : undefined;
    // Start validating the data
    // Validate value/r/s
    valueOverflowCheck({ value: tx.value, r: tx.r, s: tx.s });
    // geth limits gasLimit to 2^64-1
    valueOverflowCheck({ gasLimit: tx.gasLimit }, 64);
    // EIP-2681 limits nonce to 2^64-1 (cannot equal 2^64-1)
    valueOverflowCheck({ nonce: tx.nonce }, 64, true);
    // EIP-7825: Transaction Gas Limit Cap.
    // Under EIP-8037 the cap applies to the regular-gas dimension only (state-gas
    // is capped solely by tx.gas), so the tx-level total-gas-limit cap is lifted
    // here. The runTx-level validation enforces
    // `max(intrinsic_regular_gas, calldata_floor_gas_cost) <= TX_MAX_GAS_LIMIT`
    // instead.
    if (tx.common.isActivatedEIP(7825) && !tx.common.isActivatedEIP(8037)) {
        const maxGasLimit = tx.common.param('maxTransactionGasLimit');
        if (tx.gasLimit > maxGasLimit) {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Transaction gas limit ${tx.gasLimit} exceeds the maximum allowed by EIP-7825 (${maxGasLimit})`);
        }
    }
    const createContract = tx.to === undefined || tx.to === null;
    const allowUnlimitedInitCodeSize = opts.allowUnlimitedInitCodeSize ?? false;
    if (createContract && tx.common.isActivatedEIP(3860) && allowUnlimitedInitCodeSize === false) {
        checkMaxInitCodeSize(tx.common, tx.data.length);
    }
}
/**
 * Converts a transaction to its base JSON representation
 * @param tx - The transaction interface
 * @returns JSON object with base transaction fields
 */
function getBaseJSON(tx) {
    return {
        type: (0, util_1.bigIntToHex)(BigInt(tx.type)),
        nonce: (0, util_1.bigIntToHex)(tx.nonce),
        gasLimit: (0, util_1.bigIntToHex)(tx.gasLimit),
        to: tx.to !== undefined ? tx.to.toString() : undefined,
        value: (0, util_1.bigIntToHex)(tx.value),
        data: (0, util_1.bytesToHex)(tx.data),
        v: tx.v !== undefined ? (0, util_1.bigIntToHex)(tx.v) : undefined,
        r: tx.r !== undefined ? (0, util_1.bigIntToHex)(tx.r) : undefined,
        s: tx.s !== undefined ? (0, util_1.bigIntToHex)(tx.s) : undefined,
        chainId: (0, util_1.bigIntToHex)(tx.common.chainId()),
        yParity: tx.v === 0n || tx.v === 1n ? (0, util_1.bigIntToHex)(tx.v) : undefined,
    };
}
//# sourceMappingURL=internal.js.map