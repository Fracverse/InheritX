import { BinaryParser } from '../serdes/binary-parser';
import { SerializedType } from './serialized-type';
/**
 * STNumber: Encodes XRPL's "Number" type.
 *
 * - Always encoded as 12 bytes: 8-byte signed mantissa, 4-byte signed exponent, both big-endian.
 * - Can only be constructed from a valid number string or another STNumber instance.
 *
 * Usage:
 *   STNumber.from("1.2345e5")
 *   STNumber.from("-123")
 *   STNumber.fromParser(parser)
 */
export declare class STNumber extends SerializedType {
    /** 12 zero bytes, represents canonical zero. */
    static defaultBytes: Uint8Array;
    /**
     * Construct a STNumber from 12 bytes (8 for mantissa, 4 for exponent).
     * @param bytes - 12-byte Uint8Array
     * @throws Error if input is not a Uint8Array of length 12.
     */
    constructor(bytes?: Uint8Array);
    /**
     * Construct from a number string (or another STNumber).
     *
     * @param value - A string, or STNumber instance.
     * @returns STNumber instance.
     * @throws Error if not a string or STNumber.
     */
    static from(value: unknown): STNumber;
    /**
     * Construct from a number string (integer, decimal, or scientific notation).
     * Handles normalization to XRPL Number constraints.
     *
     * @param val - The number as a string (e.g. '1.23', '-123e5').
     * @returns STNumber instance
     * @throws Error if val is not a valid number string.
     */
    static fromValue(val: string): STNumber;
    /**
     * Read a STNumber from a BinaryParser stream (12 bytes).
     * @param parser - BinaryParser positioned at the start of a number
     * @returns STNumber instance
     */
    static fromParser(parser: BinaryParser): STNumber;
    /**
     * Convert this STNumber to a normalized string representation.
     * The output is decimal or scientific notation, depending on exponent range.
     * Follows XRPL convention: zero is "0", other values are normalized to a canonical string.
     *
     * @returns String representation of the value
     */
    toJSON(): string;
}
