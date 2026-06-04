"use strict";
// Utility helpers to convert access lists from the byte format and JSON format and vice versa
Object.defineProperty(exports, "__esModule", { value: true });
exports.accessListBytesToJSON = accessListBytesToJSON;
exports.accessListJSONToBytes = accessListJSONToBytes;
const util_1 = require("@ethereumjs/util");
/**
 * Converts an access list in bytes to a JSON format
 * @param accessList
 * @returns JSON format of the access list
 */
function accessListBytesToJSON(accessList) {
    return accessList.map(([address, storageSlots]) => ({
        address: (0, util_1.bytesToHex)((0, util_1.setLengthLeft)(address, 20)),
        storageKeys: storageSlots.map((slot) => (0, util_1.bytesToHex)((0, util_1.setLengthLeft)(slot, 32))),
    }));
}
/**
 * Converts an access list in JSON to a bytes format
 * @param accessList
 * @returns bytes format of the access list
 */
function accessListJSONToBytes(accessList) {
    return accessList.map((item) => [
        (0, util_1.hexToBytes)(item.address),
        item.storageKeys.map((key) => (0, util_1.hexToBytes)(key)),
    ]);
}
//# sourceMappingURL=access.js.map