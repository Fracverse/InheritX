import { Comparable } from './serialized-type';
/**
 * Base class for serializing and deserializing signed integers.
 */
declare abstract class Int extends Comparable<Int | number> {
    protected static width: number;
    constructor(bytes: Uint8Array);
    /**
     * Overload of compareTo for Comparable
     *
     * @param other other Int to compare this to
     * @returns -1, 0, or 1 depending on how the objects relate to each other
     */
    compareTo(other: Int | number): number;
    /**
     * Convert an Int object to JSON
     *
     * @returns number or string represented by this.bytes
     */
    toJSON(): number | string;
    /**
     * Get the value of the Int represented by this.bytes
     *
     * @returns the value
     */
    abstract valueOf(): number | bigint;
    /**
     * Validate that a number is within the specified signed integer range
     *
     * @param typeName The name of the type (for error messages)
     * @param val The number to validate
     * @param min The minimum allowed value
     * @param max The maximum allowed value
     * @throws Error if the value is out of range
     */
    static checkIntRange(typeName: string, val: number | bigint, min: number | bigint, max: number | bigint): void;
}
export { Int };
