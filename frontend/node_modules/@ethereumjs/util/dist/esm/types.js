import { bytesToBigInt, bytesToHex, toBytes } from "./bytes.js";
import { EthereumJSErrorWithoutCode } from "./errors.js";
import { isHexString } from "./internal.js";
export function isNestedUint8Array(value) {
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
export const TypeOutput = {
    Number: 0,
    BigInt: 1,
    Uint8Array: 2,
    PrefixedHexString: 3,
};
export function toType(input, outputType) {
    if (input === null) {
        return null;
    }
    if (input === undefined) {
        return undefined;
    }
    if (typeof input === 'string' && !isHexString(input)) {
        throw EthereumJSErrorWithoutCode(`A string must be provided with a 0x-prefix, given: ${input}`);
    }
    else if (typeof input === 'number' && !Number.isSafeInteger(input)) {
        throw EthereumJSErrorWithoutCode('The provided number is greater than MAX_SAFE_INTEGER (please use an alternative input type)');
    }
    const output = toBytes(input);
    switch (outputType) {
        case TypeOutput.Uint8Array:
            return output;
        case TypeOutput.BigInt:
            return bytesToBigInt(output);
        case TypeOutput.Number: {
            const bigInt = bytesToBigInt(output);
            if (bigInt > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw EthereumJSErrorWithoutCode('The provided number is greater than MAX_SAFE_INTEGER (please use an alternative output type)');
            }
            return Number(bigInt);
        }
        case TypeOutput.PrefixedHexString:
            return bytesToHex(output);
        default:
            throw EthereumJSErrorWithoutCode('unknown outputType');
    }
}
export function isEOACode7702AuthorizationListBytes(input) {
    if (input.length === 0) {
        return true;
    }
    const firstItem = input[0];
    if (Array.isArray(firstItem)) {
        return true;
    }
    return false;
}
export function isEOACode7702AuthorizationList(input) {
    return !isEOACode7702AuthorizationListBytes(input); // This is exactly the same method, except the output is negated.
}
//# sourceMappingURL=types.js.map