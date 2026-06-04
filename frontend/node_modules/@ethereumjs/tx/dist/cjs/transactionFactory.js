"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createTx = createTx;
exports.createTxFromRLP = createTxFromRLP;
exports.createTxFromBlockBodyData = createTxFromBlockBodyData;
exports.createTxFromRPC = createTxFromRPC;
exports.createTxFromJSONRPCProvider = createTxFromJSONRPCProvider;
const util_1 = require("@ethereumjs/util");
const constructors_ts_1 = require("./1559/constructors.js");
const constructors_ts_2 = require("./2930/constructors.js");
const constructors_ts_3 = require("./4844/constructors.js");
const constructors_ts_4 = require("./7702/constructors.js");
const constructors_ts_5 = require("./legacy/constructors.js");
const types_ts_1 = require("./types.js");
const general_ts_1 = require("./util/general.js");
/**
 * Create a transaction from a `txData` object
 *
 * @param txData - The transaction data. The `type` field will determine which transaction type is returned (if undefined, creates a legacy transaction)
 * @param txOptions - Options to pass on to the constructor of the transaction
 */
function createTx(txData, txOptions = {}) {
    if (!('type' in txData) || txData.type === undefined) {
        // Assume legacy transaction
        return (0, constructors_ts_5.createLegacyTx)(txData, txOptions);
    }
    else {
        if ((0, types_ts_1.isLegacyTxData)(txData)) {
            return (0, constructors_ts_5.createLegacyTx)(txData, txOptions);
        }
        else if ((0, types_ts_1.isAccessList2930TxData)(txData)) {
            return (0, constructors_ts_2.createAccessList2930Tx)(txData, txOptions);
        }
        else if ((0, types_ts_1.isFeeMarket1559TxData)(txData)) {
            return (0, constructors_ts_1.createFeeMarket1559Tx)(txData, txOptions);
        }
        else if ((0, types_ts_1.isBlob4844TxData)(txData)) {
            return (0, constructors_ts_3.createBlob4844Tx)(txData, txOptions);
        }
        else if ((0, types_ts_1.isEOACode7702TxData)(txData)) {
            return (0, constructors_ts_4.createEOACode7702Tx)(txData, txOptions);
        }
        else {
            throw (0, util_1.EthereumJSErrorWithoutCode)(`Tx instantiation with type ${txData?.type} not supported`);
        }
    }
}
/**
 * This method tries to decode serialized data.
 *
 * @param data - The data Uint8Array
 * @param txOptions - The transaction options
 */
function createTxFromRLP(data, txOptions = {}) {
    if (data[0] <= 0x7f) {
        // Determine the type.
        switch (data[0]) {
            case types_ts_1.TransactionType.AccessListEIP2930:
                return (0, constructors_ts_2.createAccessList2930TxFromRLP)(data, txOptions);
            case types_ts_1.TransactionType.FeeMarketEIP1559:
                return (0, constructors_ts_1.createFeeMarket1559TxFromRLP)(data, txOptions);
            case types_ts_1.TransactionType.BlobEIP4844:
                return (0, constructors_ts_3.createBlob4844TxFromRLP)(data, txOptions);
            case types_ts_1.TransactionType.EOACodeEIP7702:
                return (0, constructors_ts_4.createEOACode7702TxFromRLP)(data, txOptions);
            default:
                throw (0, util_1.EthereumJSErrorWithoutCode)(`TypedTransaction with ID ${data[0]} unknown`);
        }
    }
    else {
        return (0, constructors_ts_5.createLegacyTxFromRLP)(data, txOptions);
    }
}
/**
 * When decoding a BlockBody, in the transactions field, a field is either:
 * A Uint8Array (a TypedTransaction - encoded as TransactionType || rlp(TransactionPayload))
 * A Uint8Array[] (Legacy Transaction)
 * This method returns the right transaction.
 *
 * @param data - A Uint8Array or Uint8Array[]
 * @param txOptions - The transaction options
 */
function createTxFromBlockBodyData(data, txOptions = {}) {
    if (data instanceof Uint8Array) {
        return createTxFromRLP(data, txOptions);
    }
    else if (Array.isArray(data)) {
        // It is a legacy transaction
        return (0, constructors_ts_5.createLegacyTxFromBytesArray)(data, txOptions);
    }
    else {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Cannot decode transaction: unknown type input');
    }
}
/**
 * Method to decode data retrieved from RPC, such as `eth_getTransactionByHash`
 * Note that this normalizes some of the parameters
 * @param txData The RPC-encoded data
 * @param txOptions The transaction options
 * @returns A promise that resolves with the instantiated transaction
 */
async function createTxFromRPC(txData, txOptions = {}) {
    return createTx((0, general_ts_1.normalizeTxParams)(txData), txOptions);
}
/**
 *  Method to retrieve a transaction from the provider
 * @param provider - a url string for a JSON-RPC provider or an Ethers JSONRPCProvider object
 * @param txHash - Transaction hash
 * @param txOptions - The transaction options
 * @returns the transaction specified by `txHash`
 */
async function createTxFromJSONRPCProvider(provider, txHash, txOptions) {
    const prov = (0, util_1.getProvider)(provider);
    const txData = await (0, util_1.fetchFromProvider)(prov, {
        method: 'eth_getTransactionByHash',
        params: [txHash],
    });
    if (txData === null) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('No data returned from provider');
    }
    return createTxFromRPC(txData, txOptions);
}
//# sourceMappingURL=transactionFactory.js.map