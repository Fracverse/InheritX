"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compare = exports.equal = exports.readInt64BE = exports.readInt32BE = exports.readUInt32BE = exports.readUInt16BE = exports.writeInt64BE = exports.writeInt32BE = exports.writeUInt32BE = exports.writeUInt16BE = exports.writeUInt8 = void 0;
/**
 * Writes value to array at the specified offset. The value must be a valid unsigned 8-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
function writeUInt8(array, value, offset) {
    value = Number(value);
    array[offset] = value;
}
exports.writeUInt8 = writeUInt8;
/**
 * Writes value to array at the specified offset as big-endian. The value must be a valid unsigned 16-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
function writeUInt16BE(array, value, offset) {
    value = Number(value);
    array[offset] = value >>> 8;
    array[offset + 1] = value;
}
exports.writeUInt16BE = writeUInt16BE;
/**
 * Writes value to array at the specified offset as big-endian. The value must be a valid unsigned 32-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
function writeUInt32BE(array, value, offset) {
    array[offset] = (value >>> 24) & 0xff;
    array[offset + 1] = (value >>> 16) & 0xff;
    array[offset + 2] = (value >>> 8) & 0xff;
    array[offset + 3] = value & 0xff;
}
exports.writeUInt32BE = writeUInt32BE;
/**
 * Writes a signed 32-bit integer to a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to write to.
 * @param value - The signed 32-bit integer to write.
 * @param offset - The offset at which to write.
 */
function writeInt32BE(array, value, offset) {
    new DataView(array.buffer, array.byteOffset, array.byteLength).setInt32(offset, value, false);
}
exports.writeInt32BE = writeInt32BE;
/**
 * Writes a signed 64-bit integer (BigInt) to a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to write to.
 * @param value - The signed 64-bit integer (BigInt) to write.
 * @param offset - The offset at which to write.
 */
function writeInt64BE(array, value, offset) {
    new DataView(array.buffer, array.byteOffset, array.byteLength).setBigInt64(offset, value, false);
}
exports.writeInt64BE = writeInt64BE;
/**
 * Reads an unsigned, big-endian 16-bit integer from the array at the specified offset.
 * @param array Uint8Array to read
 * @param offset Number of bytes to skip before starting to read. Must satisfy 0 <= offset <= buf.length - 2
 */
function readUInt16BE(array, offset) {
    return new DataView(array.buffer).getUint16(offset, false).toString(10);
}
exports.readUInt16BE = readUInt16BE;
/**
 * Reads an unsigned, big-endian 16-bit integer from the array at the specified offset.
 * @param array Uint8Array to read
 * @param offset Number of bytes to skip before starting to read. Must satisfy 0 <= offset <= buf.length - 4
 */
function readUInt32BE(array, offset) {
    return new DataView(array.buffer).getUint32(offset, false).toString(10);
}
exports.readUInt32BE = readUInt32BE;
/**
 * Reads a signed 32-bit integer from a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to read from.
 * @param offset - The offset at which to start reading.
 * @returns The signed 32-bit integer.
 */
function readInt32BE(array, offset) {
    return new DataView(array.buffer, array.byteOffset, array.byteLength).getInt32(offset, false);
}
exports.readInt32BE = readInt32BE;
/**
 * Reads a signed 64-bit integer (BigInt) from a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to read from.
 * @param offset - The offset at which to start reading.
 * @returns The signed 64-bit integer (BigInt).
 */
function readInt64BE(array, offset) {
    return new DataView(array.buffer, array.byteOffset, array.byteLength).getBigInt64(offset, false);
}
exports.readInt64BE = readInt64BE;
/**
 * Compares two Uint8Array or ArrayBuffers
 * @param a first array to compare
 * @param b second array to compare
 */
function equal(a, b) {
    const aUInt = a instanceof ArrayBuffer ? new Uint8Array(a, 0) : a;
    const bUInt = b instanceof ArrayBuffer ? new Uint8Array(b, 0) : b;
    if (aUInt.byteLength != bUInt.byteLength)
        return false;
    if (aligned32(aUInt) && aligned32(bUInt))
        return compare32(aUInt, bUInt) === 0;
    if (aligned16(aUInt) && aligned16(bUInt))
        return compare16(aUInt, bUInt) === 0;
    return compare8(aUInt, bUInt) === 0;
}
exports.equal = equal;
/**
 * Compares two 8 bit aligned arrays
 * @param a first array to compare
 * @param b second array to compare
 */
function compare8(a, b) {
    const ua = new Uint8Array(a.buffer, a.byteOffset, a.byteLength);
    const ub = new Uint8Array(b.buffer, b.byteOffset, b.byteLength);
    return compare(ua, ub);
}
/**
 * Compares two 16 bit aligned arrays
 * @param a first array to compare
 * @param b second array to compare
 */
function compare16(a, b) {
    const ua = new Uint16Array(a.buffer, a.byteOffset, a.byteLength / 2);
    const ub = new Uint16Array(b.buffer, b.byteOffset, b.byteLength / 2);
    return compare(ua, ub);
}
/**
 * Compares two 32 bit aligned arrays
 * @param a first array to compare
 * @param b second array to compare
 */
function compare32(a, b) {
    const ua = new Uint32Array(a.buffer, a.byteOffset, a.byteLength / 4);
    const ub = new Uint32Array(b.buffer, b.byteOffset, b.byteLength / 4);
    return compare(ua, ub);
}
/**
 * Compare two TypedArrays
 * @param a first array to compare
 * @param b second array to compare
 */
function compare(a, b) {
    if (a.byteLength !== b.byteLength) {
        throw new Error('Cannot compare arrays of different length');
    }
    for (let i = 0; i < a.length - 1; i += 1) {
        if (a[i] > b[i])
            return 1;
        if (a[i] < b[i])
            return -1;
    }
    return 0;
}
exports.compare = compare;
/**
 * Determine if TypedArray is 16 bit aligned
 * @param array The array to check
 */
function aligned16(array) {
    return array.byteOffset % 2 === 0 && array.byteLength % 2 === 0;
}
/**
 * Determine if TypedArray is 32 bit aligned
 * @param array The array to check
 */
function aligned32(array) {
    return array.byteOffset % 4 === 0 && array.byteLength % 4 === 0;
}
//# sourceMappingURL=utils.js.map