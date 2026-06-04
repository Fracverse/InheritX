"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UInt64 = void 0;
const uint_1 = require("./uint");
const utils_1 = require("@xrplf/isomorphic/utils");
const utils_2 = require("../utils");
const enums_1 = require("../enums");
const HEX_REGEX = /^[a-fA-F0-9]{1,16}$/;
const BASE10_REGEX = /^[0-9]{1,20}$/;
const mask = BigInt(0x00000000ffffffff);
const BASE10_AMOUNT_FIELDS = new Set([
    'MaximumAmount',
    'OutstandingAmount',
    'MPTAmount',
    'LockedAmount',
]);
function isBase10(fieldName) {
    return BASE10_AMOUNT_FIELDS.has(fieldName);
}
/**
 * Derived UInt class for serializing/deserializing 64 bit UInt
 */
class UInt64 extends uint_1.UInt {
    constructor(bytes) {
        super(bytes !== null && bytes !== void 0 ? bytes : UInt64.defaultUInt64.bytes);
    }
    static fromParser(parser) {
        return new UInt64(parser.read(UInt64.width));
    }
    /**
     * Construct a UInt64 object
     *
     * @param val A UInt64, hex-string, bigInt, or number
     * @returns A UInt64 object
     */
    // eslint-disable-next-line complexity
    static from(val, fieldName = '') {
        if (val instanceof UInt64) {
            return val;
        }
        let buf = new Uint8Array(UInt64.width);
        if (typeof val === 'number' && Number.isInteger(val)) {
            if (val < 0) {
                throw new Error('value must be an unsigned integer');
            }
            const number = BigInt(val);
            const intBuf = [new Uint8Array(4), new Uint8Array(4)];
            (0, utils_2.writeUInt32BE)(intBuf[0], Number(number >> BigInt(32)), 0);
            (0, utils_2.writeUInt32BE)(intBuf[1], Number(number & BigInt(mask)), 0);
            return new UInt64((0, utils_1.concat)(intBuf));
        }
        if (typeof val === 'string') {
            if (isBase10(fieldName)) {
                if (!BASE10_REGEX.test(val)) {
                    throw new Error(`${fieldName} ${val} is not a valid base 10 string`);
                }
                val = BigInt(val).toString(16);
            }
            if (typeof val === 'string' && !HEX_REGEX.test(val)) {
                throw new Error(`${val} is not a valid hex-string`);
            }
            const strBuf = val.padStart(16, '0');
            buf = (0, utils_1.hexToBytes)(strBuf);
            return new UInt64(buf);
        }
        if (typeof val === 'bigint') {
            const intBuf = [new Uint8Array(4), new Uint8Array(4)];
            (0, utils_2.writeUInt32BE)(intBuf[0], Number(Number(val >> BigInt(32))), 0);
            (0, utils_2.writeUInt32BE)(intBuf[1], Number(val & BigInt(mask)), 0);
            return new UInt64((0, utils_1.concat)(intBuf));
        }
        throw new Error('Cannot construct UInt64 from given value');
    }
    /**
     * The JSON representation of a UInt64 object
     *
     * @returns a hex-string
     */
    toJSON(_definitions = enums_1.DEFAULT_DEFINITIONS, fieldName = '') {
        const hexString = (0, utils_1.bytesToHex)(this.bytes);
        if (isBase10(fieldName)) {
            return BigInt('0x' + hexString).toString(10);
        }
        return hexString;
    }
    /**
     * Get the value of the UInt64
     *
     * @returns the number represented buy this.bytes
     */
    valueOf() {
        const msb = BigInt((0, utils_2.readUInt32BE)(this.bytes.slice(0, 4), 0));
        const lsb = BigInt((0, utils_2.readUInt32BE)(this.bytes.slice(4), 0));
        return (msb << BigInt(32)) | lsb;
    }
    /**
     * Get the bytes representation of the UInt64 object
     *
     * @returns 8 bytes representing the UInt64
     */
    toBytes() {
        return this.bytes;
    }
}
exports.UInt64 = UInt64;
UInt64.width = 64 / 8; // 8
UInt64.defaultUInt64 = new UInt64(new Uint8Array(UInt64.width));
//# sourceMappingURL=uint-64.js.map