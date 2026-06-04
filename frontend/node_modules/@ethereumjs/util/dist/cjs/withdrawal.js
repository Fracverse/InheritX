"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Withdrawal = void 0;
exports.withdrawalToBytesArray = withdrawalToBytesArray;
exports.createWithdrawal = createWithdrawal;
exports.createWithdrawalFromBytesArray = createWithdrawalFromBytesArray;
const address_ts_1 = require("./address.js");
const bytes_ts_1 = require("./bytes.js");
const constants_ts_1 = require("./constants.js");
const types_ts_1 = require("./types.js");
/**
 * Convert a withdrawal to a byte array
 * @param withdrawal the withdrawal to convert
 * @returns byte array of the withdrawal
 */
function withdrawalToBytesArray(withdrawal) {
    const { index, validatorIndex, address, amount } = withdrawal;
    const indexBytes = (0, types_ts_1.toType)(index, types_ts_1.TypeOutput.BigInt) === constants_ts_1.BIGINT_0
        ? new Uint8Array()
        : (0, types_ts_1.toType)(index, types_ts_1.TypeOutput.Uint8Array);
    const validatorIndexBytes = (0, types_ts_1.toType)(validatorIndex, types_ts_1.TypeOutput.BigInt) === constants_ts_1.BIGINT_0
        ? new Uint8Array()
        : (0, types_ts_1.toType)(validatorIndex, types_ts_1.TypeOutput.Uint8Array);
    const addressBytes = address instanceof address_ts_1.Address ? address.bytes : (0, types_ts_1.toType)(address, types_ts_1.TypeOutput.Uint8Array);
    const amountBytes = (0, types_ts_1.toType)(amount, types_ts_1.TypeOutput.BigInt) === constants_ts_1.BIGINT_0
        ? new Uint8Array()
        : (0, types_ts_1.toType)(amount, types_ts_1.TypeOutput.Uint8Array);
    return [indexBytes, validatorIndexBytes, addressBytes, amountBytes];
}
/**
 * Representation of EIP-4895 withdrawal data
 */
class Withdrawal {
    /**
     * This constructor assigns and validates the values.
     * Use the static factory methods to assist in creating a Withdrawal object from varying data types.
     * Its amount is in Gwei to match CL representation and for eventual ssz withdrawalsRoot
     */
    constructor(index, validatorIndex, address, amount) {
        this.index = index;
        this.validatorIndex = validatorIndex;
        this.address = address;
        this.amount = amount;
    }
    raw() {
        return withdrawalToBytesArray(this);
    }
    toValue() {
        return {
            index: this.index,
            validatorIndex: this.validatorIndex,
            address: this.address.bytes,
            amount: this.amount,
        };
    }
    toJSON() {
        return {
            index: (0, bytes_ts_1.bigIntToHex)(this.index),
            validatorIndex: (0, bytes_ts_1.bigIntToHex)(this.validatorIndex),
            address: (0, bytes_ts_1.bytesToHex)(this.address.bytes),
            amount: (0, bytes_ts_1.bigIntToHex)(this.amount),
        };
    }
}
exports.Withdrawal = Withdrawal;
/**
 * Creates a validator withdrawal request to be submitted to the consensus layer
 * @param withdrawalData the consensus layer index and validator index values for the
 * validator requesting the withdrawal and the address and withdrawal amount of the request
 * @returns a {@link Withdrawal} object
 */
function createWithdrawal(withdrawalData) {
    const { index: indexData, validatorIndex: validatorIndexData, address: addressData, amount: amountData, } = withdrawalData;
    const index = (0, types_ts_1.toType)(indexData, types_ts_1.TypeOutput.BigInt);
    const validatorIndex = (0, types_ts_1.toType)(validatorIndexData, types_ts_1.TypeOutput.BigInt);
    const address = addressData instanceof address_ts_1.Address ? addressData : new address_ts_1.Address((0, bytes_ts_1.toBytes)(addressData));
    const amount = (0, types_ts_1.toType)(amountData, types_ts_1.TypeOutput.BigInt);
    return new Withdrawal(index, validatorIndex, address, amount);
}
/**
 * Creates a validator withdrawal request to be submitted to the consensus layer from
 * an RLP list
 * @param withdrawalArray decoded RLP list of withdrawal data elements
 * @returns a {@link Withdrawal} object
 */
function createWithdrawalFromBytesArray(withdrawalArray) {
    if (withdrawalArray.length !== 4) {
        throw Error(`Invalid withdrawalArray length expected=4 actual=${withdrawalArray.length}`);
    }
    const [index, validatorIndex, address, amount] = withdrawalArray;
    return createWithdrawal({ index, validatorIndex, address, amount });
}
//# sourceMappingURL=withdrawal.js.map