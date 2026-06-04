"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TypeOutput = void 0;
exports.isNestedUint8Array = isNestedUint8Array;
exports.toType = toType;
exports.isEOACode7702AuthorizationListBytes = isEOACode7702AuthorizationListBytes;
exports.isEOACode7702AuthorizationList = isEOACode7702AuthorizationList;
const bytes_ts_1 = require("./bytes.js");
const errors_ts_1 = require("./errors.js");
const internal_ts_1 = require("./internal.js");
function isNestedUint8Array(value) {
    if (!Array.isArray(value)) {
        return false;
    }
    for (const item of value) {
        if (Array.isArray(item)) {
            if (!isNestedUint8Array(item)) {
                return false;
            }
        }
        else if (!(item instanceof Uint8Array)) {
            return false;
        }
    }
    return true;
}
exports.TypeOutput = {
    Number: 0,
    BigInt: 1,
    Uint8Array: 2,
    PrefixedHexString: 3,
};
function toType(input, outputType) {
    if (input === null) {
        return null;
    }
    if (input === undefined) {
        return undefined;
    }
    if (typeof input === 'string' && !(0, internal_ts_1.isHexString)(input)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`A string must be provided with a 0x-prefix, given: ${input}`);
    }
    else if (typeof input === 'number' && !Number.isSafeInteger(input)) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('The provided number is greater than MAX_SAFE_INTEGER (please use an alternative input type)');
    }
    const output = (0, bytes_ts_1.toBytes)(input);
    switch (outputType) {
        case exports.TypeOutput.Uint8Array:
            return output;
        case exports.TypeOutput.BigInt:
            return (0, bytes_ts_1.bytesToBigInt)(output);
        case exports.TypeOutput.Number: {
            const bigInt = (0, bytes_ts_1.bytesToBigInt)(output);
            if (bigInt > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('The provided number is greater than MAX_SAFE_INTEGER (please use an alternative output type)');
            }
            return Number(bigInt);
        }
        case exports.TypeOutput.PrefixedHexString:
            return (0, bytes_ts_1.bytesToHex)(output);
        default:
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('unknown outputType');
    }
}
function isEOACode7702AuthorizationListBytes(input) {
    if (input.length === 0) {
        return true;
    }
    const firstItem = input[0];
    if (Array.isArray(firstItem)) {
        return true;
    }
    return false;
}
function isEOACode7702AuthorizationList(input) {
    return !isEOACode7702AuthorizationListBytes(input); // This is exactly the same method, except the output is negated.
}
//# sourceMappingURL=types.js.map