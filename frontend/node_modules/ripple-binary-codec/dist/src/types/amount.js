"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Amount = void 0;
const binary_parser_1 = require("../serdes/binary-parser");
const account_id_1 = require("./account-id");
const currency_1 = require("./currency");
const serialized_type_1 = require("./serialized-type");
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const utils_1 = require("@xrplf/isomorphic/utils");
const utils_2 = require("../utils");
const hash_192_1 = require("./hash-192");
/**
 * Constants for validating amounts
 */
const MIN_IOU_EXPONENT = -96;
const MAX_IOU_EXPONENT = 80;
const MAX_IOU_PRECISION = 16;
const MAX_DROPS = new bignumber_js_1.default('1e17');
const MIN_XRP = new bignumber_js_1.default('1e-6');
const mask = BigInt(0x00000000ffffffff);
const mptMask = BigInt(0x8000000000000000);
/**
 * BigNumber configuration for Amount IOUs
 */
bignumber_js_1.default.config({
    EXPONENTIAL_AT: [
        MIN_IOU_EXPONENT - MAX_IOU_PRECISION,
        MAX_IOU_EXPONENT + MAX_IOU_PRECISION,
    ],
});
/**
 * Type guard for AmountObjectIOU
 */
function isAmountObjectIOU(arg) {
    const keys = Object.keys(arg).sort();
    return (keys.length === 3 &&
        keys[0] === 'currency' &&
        keys[1] === 'issuer' &&
        keys[2] === 'value');
}
/**
 * Type guard for AmountObjectMPT
 */
function isAmountObjectMPT(arg) {
    const keys = Object.keys(arg).sort();
    return (keys.length === 2 && keys[0] === 'mpt_issuance_id' && keys[1] === 'value');
}
/**
 * Class for serializing/Deserializing Amounts
 */
class Amount extends serialized_type_1.SerializedType {
    constructor(bytes) {
        super(bytes !== null && bytes !== void 0 ? bytes : Amount.defaultAmount.bytes);
    }
    /**
     * Construct an amount from an IOU, MPT or string amount
     *
     * @param value An Amount, object representing an IOU, or a string
     *     representing an integer amount
     * @returns An Amount object
     */
    static from(value) {
        if (value instanceof Amount) {
            return value;
        }
        let amount = new Uint8Array(8);
        if (typeof value === 'string') {
            Amount.assertXrpIsValid(value);
            const number = BigInt(value);
            const intBuf = [new Uint8Array(4), new Uint8Array(4)];
            (0, utils_2.writeUInt32BE)(intBuf[0], Number(number >> BigInt(32)), 0);
            (0, utils_2.writeUInt32BE)(intBuf[1], Number(number & BigInt(mask)), 0);
            amount = (0, utils_1.concat)(intBuf);
            amount[0] |= 0x40;
            return new Amount(amount);
        }
        if (isAmountObjectIOU(value)) {
            const number = new bignumber_js_1.default(value.value);
            Amount.assertIouIsValid(number);
            if (number.isZero()) {
                amount[0] |= 0x80;
            }
            else {
                const integerNumberString = number
                    .times(`1e${-((number.e || 0) - 15)}`)
                    .abs()
                    .toString();
                const num = BigInt(integerNumberString);
                const intBuf = [new Uint8Array(4), new Uint8Array(4)];
                (0, utils_2.writeUInt32BE)(intBuf[0], Number(num >> BigInt(32)), 0);
                (0, utils_2.writeUInt32BE)(intBuf[1], Number(num & BigInt(mask)), 0);
                amount = (0, utils_1.concat)(intBuf);
                amount[0] |= 0x80;
                if (number.gt(new bignumber_js_1.default(0))) {
                    amount[0] |= 0x40;
                }
                const exponent = (number.e || 0) - 15;
                const exponentByte = 97 + exponent;
                amount[0] |= exponentByte >>> 2;
                amount[1] |= (exponentByte & 0x03) << 6;
            }
            const currency = currency_1.Currency.from(value.currency).toBytes();
            const issuer = account_id_1.AccountID.from(value.issuer).toBytes();
            return new Amount((0, utils_1.concat)([amount, currency, issuer]));
        }
        if (isAmountObjectMPT(value)) {
            Amount.assertMptIsValid(value.value);
            let leadingByte = new Uint8Array(1);
            leadingByte[0] |= 0x60;
            const num = BigInt(value.value);
            const intBuf = [new Uint8Array(4), new Uint8Array(4)];
            (0, utils_2.writeUInt32BE)(intBuf[0], Number(num >> BigInt(32)), 0);
            (0, utils_2.writeUInt32BE)(intBuf[1], Number(num & BigInt(mask)), 0);
            amount = (0, utils_1.concat)(intBuf);
            const mptIssuanceID = hash_192_1.Hash192.from(value.mpt_issuance_id).toBytes();
            return new Amount((0, utils_1.concat)([leadingByte, amount, mptIssuanceID]));
        }
        throw new Error('Invalid type to construct an Amount');
    }
    /**
     * Read an amount from a BinaryParser
     *
     * @param parser BinaryParser to read the Amount from
     * @returns An Amount object
     */
    static fromParser(parser) {
        const isIOU = parser.peek() & 0x80;
        if (isIOU)
            return new Amount(parser.read(48));
        // the amount can be either MPT or XRP at this point
        const isMPT = parser.peek() & 0x20;
        const numBytes = isMPT ? 33 : 8;
        return new Amount(parser.read(numBytes));
    }
    /**
     * Get the JSON representation of this Amount
     *
     * @returns the JSON interpretation of this.bytes
     */
    toJSON() {
        if (this.isNative()) {
            const bytes = this.bytes;
            const isPositive = bytes[0] & 0x40;
            const sign = isPositive ? '' : '-';
            bytes[0] &= 0x3f;
            const msb = BigInt((0, utils_2.readUInt32BE)(bytes.slice(0, 4), 0));
            const lsb = BigInt((0, utils_2.readUInt32BE)(bytes.slice(4), 0));
            const num = (msb << BigInt(32)) | lsb;
            return `${sign}${num.toString()}`;
        }
        if (this.isIOU()) {
            const parser = new binary_parser_1.BinaryParser(this.toString());
            const mantissa = parser.read(8);
            const currency = currency_1.Currency.fromParser(parser);
            const issuer = account_id_1.AccountID.fromParser(parser);
            const b1 = mantissa[0];
            const b2 = mantissa[1];
            const isPositive = b1 & 0x40;
            const sign = isPositive ? '' : '-';
            const exponent = ((b1 & 0x3f) << 2) + ((b2 & 0xff) >> 6) - 97;
            mantissa[0] = 0;
            mantissa[1] &= 0x3f;
            const value = new bignumber_js_1.default(`${sign}0x${(0, utils_1.bytesToHex)(mantissa)}`).times(`1e${exponent}`);
            Amount.assertIouIsValid(value);
            return {
                value: value.toString(),
                currency: currency.toJSON(),
                issuer: issuer.toJSON(),
            };
        }
        if (this.isMPT()) {
            const parser = new binary_parser_1.BinaryParser(this.toString());
            const leadingByte = parser.read(1);
            const amount = parser.read(8);
            const mptID = hash_192_1.Hash192.fromParser(parser);
            const isPositive = leadingByte[0] & 0x40;
            const sign = isPositive ? '' : '-';
            const msb = BigInt((0, utils_2.readUInt32BE)(amount.slice(0, 4), 0));
            const lsb = BigInt((0, utils_2.readUInt32BE)(amount.slice(4), 0));
            const num = (msb << BigInt(32)) | lsb;
            return {
                value: `${sign}${num.toString()}`,
                mpt_issuance_id: mptID.toString(),
            };
        }
        throw new Error('Invalid amount to construct JSON');
    }
    /**
     * Validate XRP amount
     *
     * @param amount String representing XRP amount
     * @returns void, but will throw if invalid amount
     */
    static assertXrpIsValid(amount) {
        if (amount.indexOf('.') !== -1) {
            throw new Error(`${amount.toString()} is an illegal amount`);
        }
        const decimal = new bignumber_js_1.default(amount);
        if (!decimal.isZero()) {
            if (decimal.lt(MIN_XRP) || decimal.gt(MAX_DROPS)) {
                throw new Error(`${amount.toString()} is an illegal amount`);
            }
        }
    }
    /**
     * Validate IOU.value amount
     *
     * @param decimal BigNumber object representing IOU.value
     * @returns void, but will throw if invalid amount
     */
    static assertIouIsValid(decimal) {
        if (!decimal.isZero()) {
            const p = decimal.precision();
            const e = (decimal.e || 0) - 15;
            if (p > MAX_IOU_PRECISION ||
                e > MAX_IOU_EXPONENT ||
                e < MIN_IOU_EXPONENT) {
                throw new Error('Decimal precision out of range');
            }
            this.verifyNoDecimal(decimal);
        }
    }
    /**
     * Validate MPT.value amount
     *
     * @param decimal BigNumber object representing MPT.value
     * @returns void, but will throw if invalid amount
     */
    static assertMptIsValid(amount) {
        if (amount.indexOf('.') !== -1) {
            throw new Error(`${amount.toString()} is an illegal amount`);
        }
        const decimal = new bignumber_js_1.default(amount);
        if (!decimal.isZero()) {
            if (decimal < (0, bignumber_js_1.default)(0)) {
                throw new Error(`${amount.toString()} is an illegal amount`);
            }
            if (Number(BigInt(amount) & BigInt(mptMask)) != 0) {
                throw new Error(`${amount.toString()} is an illegal amount`);
            }
        }
    }
    /**
     * Ensure that the value after being multiplied by the exponent does not
     * contain a decimal.
     *
     * @param decimal a Decimal object
     * @returns a string of the object without a decimal
     */
    static verifyNoDecimal(decimal) {
        const integerNumberString = decimal
            .times(`1e${-((decimal.e || 0) - 15)}`)
            .abs()
            .toString();
        if (integerNumberString.indexOf('.') !== -1) {
            throw new Error('Decimal place found in integerNumberString');
        }
    }
    /**
     * Test if this amount is in units of Native Currency(XRP)
     *
     * @returns true if Native (XRP)
     */
    isNative() {
        return (this.bytes[0] & 0x80) === 0 && (this.bytes[0] & 0x20) === 0;
    }
    /**
     * Test if this amount is in units of MPT
     *
     * @returns true if MPT
     */
    isMPT() {
        return (this.bytes[0] & 0x80) === 0 && (this.bytes[0] & 0x20) !== 0;
    }
    /**
     * Test if this amount is in units of IOU
     *
     * @returns true if IOU
     */
    isIOU() {
        return (this.bytes[0] & 0x80) !== 0;
    }
}
exports.Amount = Amount;
Amount.defaultAmount = new Amount((0, utils_1.hexToBytes)('4000000000000000'));
//# sourceMappingURL=amount.js.map