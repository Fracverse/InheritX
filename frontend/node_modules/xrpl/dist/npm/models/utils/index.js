"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isHex = exports.hasFlag = exports.isFlagEnabled = exports.onlyHasFields = exports.INTEGER_SANITY_CHECK = void 0;
const HEX_REGEX = /^[0-9A-Fa-f]+$/u;
exports.INTEGER_SANITY_CHECK = /^[0-9]+$/u;
function onlyHasFields(obj, fields) {
    return Object.keys(obj).every((key) => fields.includes(key));
}
exports.onlyHasFields = onlyHasFields;
function isFlagEnabled(Flags, checkFlag) {
    return (BigInt(checkFlag) & BigInt(Flags)) === BigInt(checkFlag);
}
exports.isFlagEnabled = isFlagEnabled;
function hasFlag(tx, flag, flagName) {
    if (tx.Flags == null) {
        return false;
    }
    if (typeof tx.Flags === 'number') {
        return isFlagEnabled(tx.Flags, flag);
    }
    return tx.Flags[flagName] === true;
}
exports.hasFlag = hasFlag;
function isHex(str) {
    return HEX_REGEX.test(str);
}
exports.isHex = isHex;
//# sourceMappingURL=index.js.map