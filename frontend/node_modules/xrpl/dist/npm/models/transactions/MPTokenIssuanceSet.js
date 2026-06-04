"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateMPTokenIssuanceSet = exports.MPTokenIssuanceSetFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
var MPTokenIssuanceSetFlags;
(function (MPTokenIssuanceSetFlags) {
    MPTokenIssuanceSetFlags[MPTokenIssuanceSetFlags["tfMPTLock"] = 1] = "tfMPTLock";
    MPTokenIssuanceSetFlags[MPTokenIssuanceSetFlags["tfMPTUnlock"] = 2] = "tfMPTUnlock";
})(MPTokenIssuanceSetFlags || (exports.MPTokenIssuanceSetFlags = MPTokenIssuanceSetFlags = {}));
function validateMPTokenIssuanceSet(tx) {
    var _a, _b, _c;
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'MPTokenIssuanceID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Holder', common_1.isAccount);
    const flags = ((_a = tx.Flags) !== null && _a !== void 0 ? _a : 0);
    const isTfMPTLock = typeof flags === 'number'
        ? (0, utils_1.isFlagEnabled)(flags, MPTokenIssuanceSetFlags.tfMPTLock)
        : ((_b = flags.tfMPTLock) !== null && _b !== void 0 ? _b : false);
    const isTfMPTUnlock = typeof flags === 'number'
        ? (0, utils_1.isFlagEnabled)(flags, MPTokenIssuanceSetFlags.tfMPTUnlock)
        : ((_c = flags.tfMPTUnlock) !== null && _c !== void 0 ? _c : false);
    if (isTfMPTLock && isTfMPTUnlock) {
        throw new errors_1.ValidationError('MPTokenIssuanceSet: flag conflict');
    }
}
exports.validateMPTokenIssuanceSet = validateMPTokenIssuanceSet;
//# sourceMappingURL=MPTokenIssuanceSet.js.map