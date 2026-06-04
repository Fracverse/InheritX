"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.utf8ToBytes = exports.concatBytes = exports.randomBytes = exports.compareBytes = exports.intToUnpaddedBytes = exports.bigIntToAddressBytes = exports.bigIntToUnpaddedBytes = exports.bigIntMin = exports.bigIntMax = exports.bigIntToHex = exports.validateNoLeadingZeroes = exports.short = exports.addHexPrefix = exports.toUnsigned = exports.fromSigned = exports.toBytes = exports.unpadHex = exports.unpadArray = exports.unpadBytes = exports.setLengthRight = exports.setLengthLeft = exports.bigIntToBytes = exports.intToBytes = exports.intToHex = exports.bytesToInt = exports.bytesToBigInt = exports.bytesToHex = exports.unprefixedHexToBytes = exports.hexToBytes = exports.bytesToUnprefixedHex = void 0;
exports.bytesToInt32 = bytesToInt32;
exports.bytesToBigInt64 = bytesToBigInt64;
exports.int32ToBytes = int32ToBytes;
exports.bigInt64ToBytes = bigInt64ToBytes;
exports.bytesToUtf8 = bytesToUtf8;
exports.equalsBytes = equalsBytes;
exports.hexToBigInt = hexToBigInt;
exports.bytesToBits = bytesToBits;
exports.bitsToBytes = bitsToBytes;
exports.matchingBytesLength = matchingBytesLength;
exports.matchingBitsLength = matchingBitsLength;
exports.equalsBits = equalsBits;
const utils_js_1 = require("@noble/hashes/utils.js");
const errors_ts_1 = require("./errors.js");
const helpers_ts_1 = require("./helpers.js");
const internal_ts_1 = require("./internal.js");
const BIGINT_0 = BigInt(0);
/**
 * @deprecated
 */
exports.bytesToUnprefixedHex = utils_js_1.bytesToHex;
/**
 * Converts a {@link PrefixedHexString} to a {@link Uint8Array}
 * @param {PrefixedHexString} hex The 0x-prefixed hex string to convert
 * @returns {Uint8Array} The converted bytes
 * @throws If the input is not a valid 0x-prefixed hex string
 */
const hexToBytes = (hex) => {
    if (!hex.startsWith('0x'))
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('input string must be 0x prefixed');
    return (0, utils_js_1.hexToBytes)((0, internal_ts_1.padToEven)((0, internal_ts_1.stripHexPrefix)(hex)));
};
exports.hexToBytes = hexToBytes;
const unprefixedHexToBytes = (hex) => {
    if (hex.startsWith('0x'))
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('input string cannot be 0x prefixed');
    return (0, utils_js_1.hexToBytes)((0, internal_ts_1.padToEven)(hex));
};
exports.unprefixedHexToBytes = unprefixedHexToBytes;
/**
 * Converts a {@link Uint8Array} to a {@link PrefixedHexString}
 * @param {Uint8Array} bytes the bytes to convert
 * @returns {PrefixedHexString} the hex string
 * @dev Returns `0x` if provided an empty Uint8Array
 */
const bytesToHex = (bytes) => {
    // Using deprecated bytesToUnprefixedHex for performance: bytesToHex is a wrapper that adds the 0x prefix.
    // Using bytesToUnprefixedHex directly avoids creating an intermediate prefixed string and then stripping it.
    const unprefixedHex = (0, exports.bytesToUnprefixedHex)(bytes);
    return `0x${unprefixedHex}`;
};
exports.bytesToHex = bytesToHex;
// BigInt cache for the numbers 0 - 256*256-1 (two-byte bytes)
const BIGINT_CACHE = [];
for (let i = 0; i <= 256 * 256 - 1; i++) {
    BIGINT_CACHE[i] = BigInt(i);
}
/**
 * Converts a {@link Uint8Array} to a {@link bigint}
 * @param {Uint8Array} bytes the bytes to convert
 * @returns {bigint}
 */
const bytesToBigInt = (bytes, littleEndian = false) => {
    (0, helpers_ts_1.assertIsBytes)(bytes);
    if (littleEndian) {
        bytes = bytes.slice().reverse();
    }
    const hex = (0, exports.bytesToHex)(bytes);
    if (hex === '0x') {
        return BIGINT_0;
    }
    if (hex.length === 4) {
        // If the byte length is 1 (this is faster than checking `bytes.length === 1`)
        return BIGINT_CACHE[bytes[0]];
    }
    if (hex.length === 6) {
        return BIGINT_CACHE[bytes[0] * 256 + bytes[1]];
    }
    return BigInt(hex);
};
exports.bytesToBigInt = bytesToBigInt;
/**
 * Converts a {@link Uint8Array} to a {@link number}.
 * @param {Uint8Array} bytes the bytes to convert
 * @return  {number}
 * @throws If the input number exceeds 53 bits.
 */
const bytesToInt = (bytes) => {
    const res = Number((0, exports.bytesToBigInt)(bytes));
    if (!Number.isSafeInteger(res))
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('Number exceeds 53 bits');
    return res;
};
exports.bytesToInt = bytesToInt;
/******************************************/
/**
 * Converts a {@link number} into a {@link PrefixedHexString}
 * @param {number} i
 * @return {PrefixedHexString}
 */
const intToHex = (i) => {
    if (!Number.isSafeInteger(i) || i < 0) {
        throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Received an invalid integer type: ${i}`);
    }
    return `0x${i.toString(16)}`;
};
exports.intToHex = intToHex;
/**
 * Converts an {@link number} to a {@link Uint8Array}
 * @param {Number} i
 * @return {Uint8Array}
 */
const intToBytes = (i) => {
    const hex = (0, exports.intToHex)(i);
    return (0, exports.hexToBytes)(hex);
};
exports.intToBytes = intToBytes;
/**
 * Converts a {@link bigint} to a {@link Uint8Array}
 *  * @param {bigint} num the bigint to convert
 * @returns {Uint8Array}
 */
const bigIntToBytes = (num, littleEndian = false) => {
    const bytes = (0, exports.hexToBytes)(`0x${(0, internal_ts_1.padToEven)(num.toString(16))}`);
    return littleEndian ? bytes.reverse() : bytes;
};
exports.bigIntToBytes = bigIntToBytes;
/**
 * Pads a `Uint8Array` with zeros till it has `length` bytes.
 * Throws if input length exceeds target length, unless allowTruncate is true.
 * @param {Uint8Array} msg the value to pad
 * @param {number} length the number of bytes the output should be
 * @param {boolean} right whether to start padding from the left or right
 * @param {boolean} allowTruncate whether to allow truncation if msg exceeds length
 * @return {Uint8Array}
 */
const setLength = (msg, length, right, allowTruncate) => {
    if (msg.length > length) {
        if (!allowTruncate) {
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Input length ${msg.length} exceeds target length ${length}. Use allowTruncate option to truncate.`);
        }
        return right ? msg.subarray(0, length) : msg.subarray(-length);
    }
    if (msg.length < length) {
        return right
            ? new Uint8Array([...msg, ...new Uint8Array(length - msg.length)])
            : new Uint8Array([...new Uint8Array(length - msg.length), ...msg]);
    }
    return msg;
};
/**
 * Left Pads a `Uint8Array` with leading zeros till it has `length` bytes.
 * Throws if input length exceeds target length, unless allowTruncate option is true.
 * @param {Uint8Array} msg the value to pad
 * @param {number} length the number of bytes the output should be
 * @param {SetLengthOpts} opts options object with allowTruncate flag
 * @return {Uint8Array}
 */
const setLengthLeft = (msg, length, opts = {}) => {
    (0, helpers_ts_1.assertIsBytes)(msg);
    return setLength(msg, length, false, opts.allowTruncate ?? false);
};
exports.setLengthLeft = setLengthLeft;
/**
 * Right Pads a `Uint8Array` with trailing zeros till it has `length` bytes.
 * Throws if input length exceeds target length, unless allowTruncate option is true.
 * @param {Uint8Array} msg the value to pad
 * @param {number} length the number of bytes the output should be
 * @param {SetLengthOpts} opts options object with allowTruncate flag
 * @return {Uint8Array}
 */
const setLengthRight = (msg, length, opts = {}) => {
    (0, helpers_ts_1.assertIsBytes)(msg);
    return setLength(msg, length, true, opts.allowTruncate ?? false);
};
exports.setLengthRight = setLengthRight;
/**
 * Trims leading zeros from a `Uint8Array`, `number[]` or `string`.
 * @param {Uint8Array|number[]|string} a
 * @return {Uint8Array|number[]|string}
 */
const stripZeros = (a) => {
    let first = a[0];
    while (a.length > 0 && first.toString() === '0') {
        a = a.slice(1);
        first = a[0];
    }
    return a;
};
/**
 * Trims leading zeros from a `Uint8Array`.
 * @param {Uint8Array} a
 * @return {Uint8Array}
 */
const unpadBytes = (a) => {
    (0, helpers_ts_1.assertIsBytes)(a);
    return stripZeros(a);
};
exports.unpadBytes = unpadBytes;
/**
 * Trims leading zeros from an `Array` (of numbers).
 * @param  {number[]} a
 * @return {number[]}
 */
const unpadArray = (a) => {
    (0, helpers_ts_1.assertIsArray)(a);
    return stripZeros(a);
};
exports.unpadArray = unpadArray;
/**
 * Trims leading zeros from a `PrefixedHexString`.
 * @param {PrefixedHexString} a
 * @return {PrefixedHexString}
 */
const unpadHex = (a) => {
    (0, helpers_ts_1.assertIsHexString)(a);
    return `0x${stripZeros((0, internal_ts_1.stripHexPrefix)(a))}`;
};
exports.unpadHex = unpadHex;
/**
 * Attempts to turn a value into a `Uint8Array`.
 * Inputs supported: `Buffer`, `Uint8Array`, `String` (hex-prefixed), `Number`, null/undefined, `BigInt` and other objects
 * with a `toArray()` or `toBytes()` method.
 * @param {ToBytesInputTypes} v the value
 * @return {Uint8Array}
 */
const toBytes = (v) => {
    if (v === null || v === undefined) {
        return new Uint8Array();
    }
    if (Array.isArray(v) || v instanceof Uint8Array) {
        return Uint8Array.from(v);
    }
    if (typeof v === 'string') {
        if (!(0, internal_ts_1.isHexString)(v)) {
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Cannot convert string to Uint8Array. toBytes only supports 0x-prefixed hex strings and this string was given: ${v}`);
        }
        return (0, exports.hexToBytes)(v);
    }
    if (typeof v === 'number') {
        return (0, exports.intToBytes)(v);
    }
    if (typeof v === 'bigint') {
        if (v < BIGINT_0) {
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`Cannot convert negative bigint to Uint8Array. Given: ${v}`);
        }
        let n = v.toString(16);
        if (n.length % 2)
            n = '0' + n;
        return (0, exports.unprefixedHexToBytes)(n);
    }
    if (v.toBytes !== undefined) {
        // converts a `TransformableToBytes` object to a Uint8Array
        return v.toBytes();
    }
    throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('invalid type');
};
exports.toBytes = toBytes;
/**
 * Interprets a `Uint8Array` as a signed integer and returns a `BigInt`. Assumes 256-bit numbers.
 * @param {Uint8Array} num Signed integer value
 * @returns {bigint}
 */
const fromSigned = (num) => {
    return BigInt.asIntN(256, (0, exports.bytesToBigInt)(num));
};
exports.fromSigned = fromSigned;
/**
 * Converts a `BigInt` to an unsigned integer and returns it as a `Uint8Array`. Assumes 256-bit numbers.
 * @param {bigint} num
 * @returns {Uint8Array}
 */
const toUnsigned = (num) => {
    return (0, exports.bigIntToBytes)(BigInt.asUintN(256, num));
};
exports.toUnsigned = toUnsigned;
/**
 * Adds "0x" to a given `string` if it does not already start with "0x".
 * @param {string} str
 * @return {PrefixedHexString}
 */
const addHexPrefix = (str) => {
    if (typeof str !== 'string') {
        return str;
    }
    return (0, internal_ts_1.isHexString)(str) ? str : `0x${str}`;
};
exports.addHexPrefix = addHexPrefix;
/**
 * Shortens a string  or Uint8Array's hex string representation to maxLength (default 50).
 *
 * Examples:
 *
 * Input:  '657468657265756d000000000000000000000000000000000000000000000000'
 * Output: '657468657265756d0000000000000000000000000000000000…'
 * @param {Uint8Array | string} bytes
 * @param {number} maxLength
 * @return {string}
 */
const short = (bytes, maxLength = 50) => {
    const byteStr = bytes instanceof Uint8Array ? (0, exports.bytesToHex)(bytes) : bytes;
    const len = byteStr.slice(0, 2) === '0x' ? maxLength + 2 : maxLength;
    if (byteStr.length <= len) {
        return byteStr;
    }
    return byteStr.slice(0, len) + '…';
};
exports.short = short;
/**
 * Checks provided Uint8Array for leading zeroes and throws if found.
 *
 * Examples:
 *
 * Valid values: 0x1, 0x, 0x01, 0x1234
 * Invalid values: 0x0, 0x00, 0x001, 0x0001
 *
 * Note: This method is useful for validating that RLP encoded integers comply with the rule that all
 * integer values encoded to RLP must be in the most compact form and contain no leading zero bytes
 * @param values An object containing string keys and Uint8Array values
 * @throws if any provided value is found to have leading zero bytes
 */
const validateNoLeadingZeroes = (values) => {
    for (const [k, v] of Object.entries(values)) {
        if (v !== undefined && v.length > 0 && v[0] === 0) {
            throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`${k} cannot have leading zeroes, received: ${(0, exports.bytesToHex)(v)}`);
        }
    }
};
exports.validateNoLeadingZeroes = validateNoLeadingZeroes;
/**
 * Converts a {@link bigint} to a `0x` prefixed hex string
 * @param {bigint} num the bigint to convert
 * @returns {PrefixedHexString}
 */
const bigIntToHex = (num) => {
    return `0x${num.toString(16)}`;
};
exports.bigIntToHex = bigIntToHex;
/**
 * Calculates max bigint from an array of bigints
 * @param args array of bigints
 */
const bigIntMax = (...args) => args.reduce((m, e) => (e > m ? e : m));
exports.bigIntMax = bigIntMax;
/**
 * Calculates min BigInt from an array of BigInts
 * @param args array of bigints
 */
const bigIntMin = (...args) => args.reduce((m, e) => (e < m ? e : m));
exports.bigIntMin = bigIntMin;
/**
 * Convert value from bigint to an unpadded Uint8Array
 * (useful for RLP transport)
 * @param {bigint} value the bigint to convert
 * @returns {Uint8Array}
 */
const bigIntToUnpaddedBytes = (value) => {
    return (0, exports.unpadBytes)((0, exports.bigIntToBytes)(value));
};
exports.bigIntToUnpaddedBytes = bigIntToUnpaddedBytes;
const bigIntToAddressBytes = (value, strict = true) => {
    const addressBytes = (0, exports.bigIntToBytes)(value);
    if (strict && addressBytes.length > 20) {
        throw Error(`Invalid address bytes length=${addressBytes.length} strict=${strict}`);
    }
    // When not strict, allow truncation of values larger than 20 bytes
    return (0, exports.setLengthLeft)(addressBytes, 20, { allowTruncate: !strict });
};
exports.bigIntToAddressBytes = bigIntToAddressBytes;
/**
 * Convert value from number to an unpadded Uint8Array
 * (useful for RLP transport)
 * @param {number} value the bigint to convert
 * @returns {Uint8Array}
 */
const intToUnpaddedBytes = (value) => {
    return (0, exports.unpadBytes)((0, exports.intToBytes)(value));
};
exports.intToUnpaddedBytes = intToUnpaddedBytes;
/**
 * Compares two Uint8Arrays and returns a number indicating their order in a sorted array.
 *
 * @param {Uint8Array} value1 - The first Uint8Array to compare.
 * @param {Uint8Array} value2 - The second Uint8Array to compare.
 * @returns {number} A positive number if value1 is larger than value2,
 *                   A negative number if value1 is smaller than value2,
 *                   or 0 if value1 and value2 are equal.
 */
const compareBytes = (value1, value2) => {
    const bigIntValue1 = (0, exports.bytesToBigInt)(value1);
    const bigIntValue2 = (0, exports.bytesToBigInt)(value2);
    return bigIntValue1 > bigIntValue2 ? 1 : bigIntValue1 < bigIntValue2 ? -1 : 0;
};
exports.compareBytes = compareBytes;
/**
 * Generates a Uint8Array of random bytes of specified length.
 *
 * @param {number} length - The length of the Uint8Array.
 * @returns {Uint8Array} A Uint8Array of random bytes of specified length.
 */
const randomBytes = (length) => {
    return (0, utils_js_1.randomBytes)(length);
};
exports.randomBytes = randomBytes;
/**
 * This mirrors the functionality of the `ethereum-cryptography` export except
 * it skips the check to validate that every element of `arrays` is indeed a `uint8Array`
 * Can give small performance gains on large arrays
 * @param {Uint8Array[]} arrays an array of Uint8Arrays
 * @returns {Uint8Array} one Uint8Array with all the elements of the original set
 * works like `Buffer.concat`
 */
const concatBytes = (...arrays) => {
    if (arrays.length === 1)
        return arrays[0];
    const length = arrays.reduce((a, arr) => a + arr.length, 0);
    const result = new Uint8Array(length);
    for (let i = 0, pad = 0; i < arrays.length; i++) {
        const arr = arrays[i];
        result.set(arr, pad);
        pad += arr.length;
    }
    return result;
};
exports.concatBytes = concatBytes;
/**
 * @notice Convert a Uint8Array to a 32-bit integer
 * @param {Uint8Array} bytes The input Uint8Array from which to read the 32-bit integer.
 * @param {boolean} littleEndian True for little-endian, undefined or false for big-endian.
 * @return {number} The 32-bit integer read from the input Uint8Array.
 */
function bytesToInt32(bytes, littleEndian = false) {
    if (bytes.length < 4) {
        bytes = setLength(bytes, 4, littleEndian, false);
    }
    const dataView = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    return dataView.getUint32(0, littleEndian);
}
/**
 * @notice Convert a Uint8Array to a 64-bit bigint
 * @param {Uint8Array} bytes The input Uint8Array from which to read the 64-bit bigint.
 * @param {boolean} littleEndian True for little-endian, undefined or false for big-endian.
 * @return {bigint} The 64-bit bigint read from the input Uint8Array.
 */
function bytesToBigInt64(bytes, littleEndian = false) {
    if (bytes.length < 8) {
        bytes = setLength(bytes, 8, littleEndian, false);
    }
    const dataView = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    return dataView.getBigUint64(0, littleEndian);
}
/**
 * @notice Convert a 32-bit integer to a Uint8Array.
 * @param {number} value The 32-bit integer to convert.
 * @param {boolean} littleEndian True for little-endian, undefined or false for big-endian.
 * @return {Uint8Array} A Uint8Array of length 4 containing the integer.
 */
function int32ToBytes(value, littleEndian = false) {
    const buffer = new ArrayBuffer(4);
    const dataView = new DataView(buffer);
    dataView.setUint32(0, value, littleEndian);
    return new Uint8Array(buffer);
}
/**
 * @notice Convert a 64-bit bigint to a Uint8Array.
 * @param {bigint} value The 64-bit bigint to convert.
 * @param {boolean} littleEndian True for little-endian, undefined or false for big-endian.
 * @return {Uint8Array} A Uint8Array of length 8 containing the bigint.
 */
function bigInt64ToBytes(value, littleEndian = false) {
    const buffer = new ArrayBuffer(8);
    const dataView = new DataView(buffer);
    dataView.setBigUint64(0, value, littleEndian);
    return new Uint8Array(buffer);
}
var utils_js_2 = require("@noble/hashes/utils.js");
Object.defineProperty(exports, "utf8ToBytes", { enumerable: true, get: function () { return utils_js_2.utf8ToBytes; } });
/**
 * @notice Converts a Uint8Array to a UTF-8 string.
 * Implementation copied from ethereum-cryptography https://github.com/ethereum/js-ethereum-cryptography/blob/31f980b2847545d33268f2510ba38a3836202a44/src/utils.ts#L22-L27
 * @param {Uint8Array} bytes - The input Uint8Array to convert.
 * @returns {string} The UTF-8 string.
 * @throws {TypeError} If the input is not a Uint8Array.
 *
 */
function bytesToUtf8(bytes) {
    if (!(bytes instanceof Uint8Array)) {
        throw new TypeError(`bytesToUtf8 expected Uint8Array, got ${typeof bytes}`);
    }
    return new TextDecoder().decode(bytes);
}
/**
 * @notice Compares two Uint8Arrays and returns true if they are equal.
 * Implementation copied from ethereum-cryptography https://github.com/ethereum/js-ethereum-cryptography/blob/main/src/utils.ts#L35-L45
 * @param {Uint8Array} a - The first Uint8Array to compare.
 * @param {Uint8Array} b - The second Uint8Array to compare.
 * @returns {boolean} True if the Uint8Arrays are equal, false otherwise.
 */
function equalsBytes(a, b) {
    if (a.length !== b.length) {
        return false;
    }
    for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) {
            return false;
        }
    }
    return true;
}
function hexToBigInt(input) {
    return (0, exports.bytesToBigInt)((0, exports.hexToBytes)((0, internal_ts_1.isHexString)(input) ? input : `0x${input}`));
}
/**
 * Converts a Uint8Array of bytes into an array of bits.
 * @param {Uint8Array} bytes - The input byte array.
 * @param {number} bitLength - The number of bits to extract from the input bytes.
 * @returns {number[]} An array of bits (each 0 or 1) corresponding to the input bytes.
 */
function bytesToBits(bytes, bitLength) {
    const bits = [];
    for (let i = 0; i < (bitLength ?? bytes.length * 8); i++) {
        const byteIndex = Math.floor(i / 8);
        const bitIndex = 7 - (i % 8);
        bits.push((bytes[byteIndex] >> bitIndex) & 1);
    }
    return bits;
}
/**
 * Converts an array of bits into a Uint8Array.
 * The input bits are grouped into sets of 8, with the first bit in each group being the most significant.
 * @param {number[]} bits - The input array of bits (each should be 0 or 1). Its length should be a multiple of 8.
 * @returns {Uint8Array} A Uint8Array constructed from the input bits.
 */
function bitsToBytes(bits) {
    const numBytes = Math.ceil(bits.length / 8); // Ensure partial byte storage
    const byteData = new Uint8Array(numBytes);
    for (let i = 0; i < bits.length; i++) {
        const byteIndex = Math.floor(i / 8);
        const bitIndex = 7 - (i % 8);
        byteData[byteIndex] |= bits[i] << bitIndex;
    }
    return byteData;
}
/**
 * Compares two byte arrays and returns the count of consecutively matching items from the start.
 * @param {Uint8Array} bytes1 - The first Uint8Array to compare.
 * @param {Uint8Array} bytes2 - The second Uint8Array to compare.
 * @returns {number} The count of consecutively matching items from the start.
 */
function matchingBytesLength(bytes1, bytes2) {
    let count = 0;
    const minLength = Math.min(bytes1.length, bytes2.length);
    for (let i = 0; i < minLength; i++) {
        if (bytes1[i] === bytes2[i]) {
            count++;
        }
        else {
            // Break early if a mismatch is found
            break;
        }
    }
    return count;
}
/**
 * Compares two arrays of bits (0 or 1) and returns the count of consecutively matching bits from the start.
 * @param {number[]} bits1 - The first array of bits, in bytes or bits.
 * @param {number[]} bits2 - The second array of bits, in bytes or bits.
 * @returns {number} The count of consecutively matching bits from the start.
 */
function matchingBitsLength(bits1, bits2) {
    let count = 0;
    const minLength = Math.min(bits1.length, bits2.length);
    for (let i = 0; i < minLength; i++) {
        if (bits1[i] === bits2[i]) {
            count++;
        }
        else {
            return count;
        }
    }
    return count;
}
/**
 * Checks whether two arrays of bits are equal.
 *
 * Two arrays are considered equal if they have the same length and each corresponding element is identical.
 *
 * @param {number[]} bits1 - The first bits array.
 * @param {number[]} bits2 - The second bits array.
 * @returns {boolean} True if the arrays are equal; otherwise, false.
 */
function equalsBits(bits1, bits2) {
    if (bits1.length !== bits2.length) {
        return false;
    }
    for (let i = 0; i < bits1.length; i++) {
        if (bits1[i] !== bits2[i]) {
            return false;
        }
    }
    return true;
}
//# sourceMappingURL=bytes.js.map