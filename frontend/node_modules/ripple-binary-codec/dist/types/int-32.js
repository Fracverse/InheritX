"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Int32 = void 0;
const int_1 = require("./int");
const utils_1 = require("../utils");
/**
 * Derived Int class for serializing/deserializing signed 32-bit integers.
 */
class Int32 extends int_1.Int {
    constructor(bytes) {
        super(bytes !== null && bytes !== void 0 ? bytes : Int32.defaultInt32.bytes);
    }
    /**
     * Construct an Int32 from a BinaryParser
     *
     * @param parser BinaryParser to read Int32 from
     * @returns An Int32 object
     */
    static fromParser(parser) {
        return new Int32(parser.read(Int32.width));
    }
    /**
     * Construct an Int32 object from a number or string
     *
     * @param val Int32 object, number, or string
     * @returns An Int32 object
     */
    static from(val) {
        if (val instanceof Int32) {
            return val;
        }
        const buf = new Uint8Array(Int32.width);
        if (typeof val === 'string') {
            const num = Number(val);
            if (!Number.isFinite(num) || !Number.isInteger(num)) {
                throw new Error(`Cannot construct Int32 from string: ${val}`);
            }
            Int32.checkIntRange('Int32', num, Int32.MIN_VALUE, Int32.MAX_VALUE);
            (0, utils_1.writeInt32BE)(buf, num, 0);
            return new Int32(buf);
        }
        if (typeof val === 'number' && Number.isInteger(val)) {
            Int32.checkIntRange('Int32', val, Int32.MIN_VALUE, Int32.MAX_VALUE);
            (0, utils_1.writeInt32BE)(buf, val, 0);
            return new Int32(buf);
        }
        throw new Error('Cannot construct Int32 from given value');
    }
    /**
     * Get the value of the Int32 object
     *
     * @returns the signed 32-bit integer represented by this.bytes
     */
    valueOf() {
        return (0, utils_1.readInt32BE)(this.bytes, 0);
    }
}
exports.Int32 = Int32;
Int32.width = 32 / 8; // 4 bytes
Int32.defaultInt32 = new Int32(new Uint8Array(Int32.width));
// Signed 32-bit integer range
Int32.MIN_VALUE = -2147483648; // -2^31
Int32.MAX_VALUE = 2147483647; // 2^31 - 1
//# sourceMappingURL=int-32.js.map