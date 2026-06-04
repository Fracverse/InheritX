import { Int } from './int';
import { BinaryParser } from '../serdes/binary-parser';
/**
 * Derived Int class for serializing/deserializing signed 32-bit integers.
 */
declare class Int32 extends Int {
    protected static readonly width: number;
    static readonly defaultInt32: Int32;
    static readonly MIN_VALUE: number;
    static readonly MAX_VALUE: number;
    constructor(bytes: Uint8Array);
    /**
     * Construct an Int32 from a BinaryParser
     *
     * @param parser BinaryParser to read Int32 from
     * @returns An Int32 object
     */
    static fromParser(parser: BinaryParser): Int;
    /**
     * Construct an Int32 object from a number or string
     *
     * @param val Int32 object, number, or string
     * @returns An Int32 object
     */
    static from<T extends Int32 | number | string>(val: T): Int32;
    /**
     * Get the value of the Int32 object
     *
     * @returns the signed 32-bit integer represented by this.bytes
     */
    valueOf(): number;
}
export { Int32 };
