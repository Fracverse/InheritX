"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransactionType = exports.Capability = void 0;
exports.isAccessListBytes = isAccessListBytes;
exports.isAccessList = isAccessList;
exports.isLegacyTx = isLegacyTx;
exports.isAccessList2930Tx = isAccessList2930Tx;
exports.isFeeMarket1559Tx = isFeeMarket1559Tx;
exports.isBlob4844Tx = isBlob4844Tx;
exports.isEOACode7702Tx = isEOACode7702Tx;
exports.isLegacyTxData = isLegacyTxData;
exports.isAccessList2930TxData = isAccessList2930TxData;
exports.isFeeMarket1559TxData = isFeeMarket1559TxData;
exports.isBlob4844TxData = isBlob4844TxData;
exports.isEOACode7702TxData = isEOACode7702TxData;
const util_1 = require("@ethereumjs/util");
/**
 * Can be used in conjunction with {@link Transaction[TransactionType].supports}
 * to query on tx capabilities
 */
exports.Capability = {
    /**
     * Tx supports EIP-155 replay protection
     * See: [155](https://eips.ethereum.org/EIPS/eip-155) Replay Attack Protection EIP
     */
    EIP155ReplayProtection: 155,
    /**
     * Tx supports EIP-1559 gas fee market mechanism
     * See: [1559](https://eips.ethereum.org/EIPS/eip-1559) Fee Market EIP
     */
    EIP1559FeeMarket: 1559,
    /**
     * Tx is a typed transaction as defined in EIP-2718
     * See: [2718](https://eips.ethereum.org/EIPS/eip-2718) Transaction Type EIP
     */
    EIP2718TypedTransaction: 2718,
    /**
     * Tx supports access list generation as defined in EIP-2930
     * See: [2930](https://eips.ethereum.org/EIPS/eip-2930) Access Lists EIP
     */
    EIP2930AccessLists: 2930,
    /**
     * Tx supports setting EOA code
     * See [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702)
     */
    EIP7702EOACode: 7702,
};
/**
 * Type guard to check if input is AccessListBytes format
 * @param input - The input to check
 * @returns true if input is AccessListBytes format
 */
function isAccessListBytes(input) {
    if (input.length === 0) {
        return true;
    }
    const firstItem = input[0];
    if (Array.isArray(firstItem)) {
        return true;
    }
    return false;
}
/**
 * Type guard to check if input is AccessList format
 * @param input - The input to check
 * @returns true if input is AccessList format
 */
function isAccessList(input) {
    return !isAccessListBytes(input); // This is exactly the same method, except the output is negated.
}
exports.TransactionType = {
    Legacy: 0,
    AccessListEIP2930: 1,
    FeeMarketEIP1559: 2,
    BlobEIP4844: 3,
    EOACodeEIP7702: 4,
};
/**
 * Type guard to check if transaction is a Legacy transaction
 * @param tx - The transaction to check
 * @returns true if transaction is Legacy type
 */
function isLegacyTx(tx) {
    return tx.type === exports.TransactionType.Legacy;
}
/**
 * Type guard to check if transaction is an AccessList EIP-2930 transaction
 * @param tx - The transaction to check
 * @returns true if transaction is AccessList EIP-2930 type
 */
function isAccessList2930Tx(tx) {
    return tx.type === exports.TransactionType.AccessListEIP2930;
}
/**
 * Type guard to check if transaction is a Fee Market EIP-1559 transaction
 * @param tx - The transaction to check
 * @returns true if transaction is Fee Market EIP-1559 type
 */
function isFeeMarket1559Tx(tx) {
    return tx.type === exports.TransactionType.FeeMarketEIP1559;
}
/**
 * Type guard to check if transaction is a Blob EIP-4844 transaction
 * @param tx - The transaction to check
 * @returns true if transaction is Blob EIP-4844 type
 */
function isBlob4844Tx(tx) {
    return tx.type === exports.TransactionType.BlobEIP4844;
}
/**
 * Type guard to check if transaction is an EOA Code EIP-7702 transaction
 * @param tx - The transaction to check
 * @returns true if transaction is EOA Code EIP-7702 type
 */
function isEOACode7702Tx(tx) {
    return tx.type === exports.TransactionType.EOACodeEIP7702;
}
/**
 * Type guard to check if transaction data is Legacy transaction data
 * @param txData - The transaction data to check
 * @returns true if transaction data is Legacy type
 */
function isLegacyTxData(txData) {
    const txType = Number((0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.type)));
    return txType === exports.TransactionType.Legacy;
}
/**
 * Type guard to check if transaction data is AccessList EIP-2930 transaction data
 * @param txData - The transaction data to check
 * @returns true if transaction data is AccessList EIP-2930 type
 */
function isAccessList2930TxData(txData) {
    const txType = Number((0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.type)));
    return txType === exports.TransactionType.AccessListEIP2930;
}
/**
 * Type guard to check if transaction data is Fee Market EIP-1559 transaction data
 * @param txData - The transaction data to check
 * @returns true if transaction data is Fee Market EIP-1559 type
 */
function isFeeMarket1559TxData(txData) {
    const txType = Number((0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.type)));
    return txType === exports.TransactionType.FeeMarketEIP1559;
}
/**
 * Type guard to check if transaction data is Blob EIP-4844 transaction data
 * @param txData - The transaction data to check
 * @returns true if transaction data is Blob EIP-4844 type
 */
function isBlob4844TxData(txData) {
    const txType = Number((0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.type)));
    return txType === exports.TransactionType.BlobEIP4844;
}
/**
 * Type guard to check if transaction data is EOA Code EIP-7702 transaction data
 * @param txData - The transaction data to check
 * @returns true if transaction data is EOA Code EIP-7702 type
 */
function isEOACode7702TxData(txData) {
    const txType = Number((0, util_1.bytesToBigInt)((0, util_1.toBytes)(txData.type)));
    return txType === exports.TransactionType.EOACodeEIP7702;
}
//# sourceMappingURL=types.js.map