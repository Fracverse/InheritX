import { BinaryParser } from '../serdes/binary-parser';
import { JsonObject, SerializedType } from './serialized-type';
interface AmountObjectIOU extends JsonObject {
    value: string;
    currency: string;
    issuer: string;
}
interface AmountObjectMPT extends JsonObject {
    value: string;
    mpt_issuance_id: string;
}
/**
 * Interface for JSON objects that represent amounts
 */
type AmountObject = AmountObjectIOU | AmountObjectMPT;
/**
 * Class for serializing/Deserializing Amounts
 */
declare class Amount extends SerializedType {
    static defaultAmount: Amount;
    constructor(bytes: Uint8Array);
    /**
     * Construct an amount from an IOU, MPT or string amount
     *
     * @param value An Amount, object representing an IOU, or a string
     *     representing an integer amount
     * @returns An Amount object
     */
    static from<T extends Amount | AmountObject | string>(value: T): Amount;
    /**
     * Read an amount from a BinaryParser
     *
     * @param parser BinaryParser to read the Amount from
     * @returns An Amount object
     */
    static fromParser(parser: BinaryParser): Amount;
    /**
     * Get the JSON representation of this Amount
     *
     * @returns the JSON interpretation of this.bytes
     */
    toJSON(): AmountObject | string;
    /**
     * Validate XRP amount
     *
     * @param amount String representing XRP amount
     * @returns void, but will throw if invalid amount
     */
    private static assertXrpIsValid;
    /**
     * Validate IOU.value amount
     *
     * @param decimal BigNumber object representing IOU.value
     * @returns void, but will throw if invalid amount
     */
    private static assertIouIsValid;
    /**
     * Validate MPT.value amount
     *
     * @param decimal BigNumber object representing MPT.value
     * @returns void, but will throw if invalid amount
     */
    private static assertMptIsValid;
    /**
     * Ensure that the value after being multiplied by the exponent does not
     * contain a decimal.
     *
     * @param decimal a Decimal object
     * @returns a string of the object without a decimal
     */
    private static verifyNoDecimal;
    /**
     * Test if this amount is in units of Native Currency(XRP)
     *
     * @returns true if Native (XRP)
     */
    private isNative;
    /**
     * Test if this amount is in units of MPT
     *
     * @returns true if MPT
     */
    private isMPT;
    /**
     * Test if this amount is in units of IOU
     *
     * @returns true if IOU
     */
    private isIOU;
}
export { Amount, AmountObject };
