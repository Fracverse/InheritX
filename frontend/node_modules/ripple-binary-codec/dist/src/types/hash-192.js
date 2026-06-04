"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Hash192 = void 0;
const hash_1 = require("./hash");
/**
 * Hash with a width of 192 bits
 */
class Hash192 extends hash_1.Hash {
    constructor(bytes) {
        if ((bytes === null || bytes === void 0 ? void 0 : bytes.byteLength) === 0) {
            bytes = Hash192.ZERO_192.bytes;
        }
        super(bytes !== null && bytes !== void 0 ? bytes : Hash192.ZERO_192.bytes);
    }
}
exports.Hash192 = Hash192;
Hash192.width = 24;
Hash192.ZERO_192 = new Hash192(new Uint8Array(Hash192.width));
//# sourceMappingURL=hash-192.js.map