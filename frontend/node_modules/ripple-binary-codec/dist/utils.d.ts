/// <reference types="node" />
import TypedArray = NodeJS.TypedArray;
/**
 * Writes value to array at the specified offset. The value must be a valid unsigned 8-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
export declare function writeUInt8(array: Uint8Array, value: number, offset: number): void;
/**
 * Writes value to array at the specified offset as big-endian. The value must be a valid unsigned 16-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
export declare function writeUInt16BE(array: Uint8Array, value: number, offset: number): void;
/**
 * Writes value to array at the specified offset as big-endian. The value must be a valid unsigned 32-bit integer.
 * @param array Uint8Array to be written to
 * @param value Number to be written to array.
 * @param offset plus the number of bytes written.
 */
export declare function writeUInt32BE(array: Uint8Array, value: number, offset: number): void;
/**
 * Writes a signed 32-bit integer to a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to write to.
 * @param value - The signed 32-bit integer to write.
 * @param offset - The offset at which to write.
 */
export declare function writeInt32BE(array: Uint8Array, value: number, offset: number): void;
/**
 * Writes a signed 64-bit integer (BigInt) to a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to write to.
 * @param value - The signed 64-bit integer (BigInt) to write.
 * @param offset - The offset at which to write.
 */
export declare function writeInt64BE(array: Uint8Array, value: bigint, offset: number): void;
/**
 * Reads an unsigned, big-endian 16-bit integer from the array at the specified offset.
 * @param array Uint8Array to read
 * @param offset Number of bytes to skip before starting to read. Must satisfy 0 <= offset <= buf.length - 2
 */
export declare function readUInt16BE(array: Uint8Array, offset: number): string;
/**
 * Reads an unsigned, big-endian 16-bit integer from the array at the specified offset.
 * @param array Uint8Array to read
 * @param offset Number of bytes to skip before starting to read. Must satisfy 0 <= offset <= buf.length - 4
 */
export declare function readUInt32BE(array: Uint8Array, offset: number): string;
/**
 * Reads a signed 32-bit integer from a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to read from.
 * @param offset - The offset at which to start reading.
 * @returns The signed 32-bit integer.
 */
export declare function readInt32BE(array: Uint8Array, offset: number): number;
/**
 * Reads a signed 64-bit integer (BigInt) from a Uint8Array at the specified offset (big-endian).
 *
 * @param array - The Uint8Array to read from.
 * @param offset - The offset at which to start reading.
 * @returns The signed 64-bit integer (BigInt).
 */
export declare function readInt64BE(array: Uint8Array, offset: number): bigint;
/**
 * Compares two Uint8Array or ArrayBuffers
 * @param a first array to compare
 * @param b second array to compare
 */
export declare function equal(a: Uint8Array | ArrayBuffer, b: Uint8Array | ArrayBuffer): boolean;
/**
 * Compare two TypedArrays
 * @param a first array to compare
 * @param b second array to compare
 */
export declare function compare(a: TypedArray, b: TypedArray): 1 | -1 | 0;
