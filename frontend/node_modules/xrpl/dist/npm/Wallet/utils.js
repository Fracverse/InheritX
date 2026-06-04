"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.computeSignature = exports.getDecodedTransaction = exports.addressToBigNumber = exports.NUM_BITS_IN_HEX = exports.compareSigners = void 0;
const utils_1 = require("@xrplf/isomorphic/utils");
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const ripple_address_codec_1 = require("ripple-address-codec");
const ripple_binary_codec_1 = require("ripple-binary-codec");
const ripple_keypairs_1 = require("ripple-keypairs");
function compareSigners(left, right) {
    if (!left.Account || !right.Account) {
        throw new Error('compareSigners: Account cannot be null or undefined');
    }
    const result = addressToBigNumber(left.Account).comparedTo(addressToBigNumber(right.Account));
    if (result === null) {
        throw new Error('compareSigners: Invalid account address comparison resulted in NaN');
    }
    return result;
}
exports.compareSigners = compareSigners;
exports.NUM_BITS_IN_HEX = 16;
function addressToBigNumber(address) {
    const hex = (0, utils_1.bytesToHex)((0, ripple_address_codec_1.decodeAccountID)(address));
    return new bignumber_js_1.default(hex, exports.NUM_BITS_IN_HEX);
}
exports.addressToBigNumber = addressToBigNumber;
function getDecodedTransaction(txOrBlob) {
    if (typeof txOrBlob === 'object') {
        return (0, ripple_binary_codec_1.decode)((0, ripple_binary_codec_1.encode)(txOrBlob));
    }
    return (0, ripple_binary_codec_1.decode)(txOrBlob);
}
exports.getDecodedTransaction = getDecodedTransaction;
function computeSignature(tx, privateKey, signAs) {
    if (signAs) {
        const classicAddress = (0, ripple_address_codec_1.isValidXAddress)(signAs)
            ? (0, ripple_address_codec_1.xAddressToClassicAddress)(signAs).classicAddress
            : signAs;
        return (0, ripple_keypairs_1.sign)((0, ripple_binary_codec_1.encodeForMultisigning)(tx, classicAddress), privateKey);
    }
    return (0, ripple_keypairs_1.sign)((0, ripple_binary_codec_1.encodeForSigning)(tx), privateKey);
}
exports.computeSignature = computeSignature;
//# sourceMappingURL=utils.js.map