"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateMPTokenIssuanceCreate = exports.MPTokenIssuanceCreateFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const mptokenMetadata_1 = require("../utils/mptokenMetadata");
const common_1 = require("./common");
const MAX_AMT = '9223372036854775807';
const MAX_TRANSFER_FEE = 50000;
var MPTokenIssuanceCreateFlags;
(function (MPTokenIssuanceCreateFlags) {
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTCanLock"] = 2] = "tfMPTCanLock";
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTRequireAuth"] = 4] = "tfMPTRequireAuth";
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTCanEscrow"] = 8] = "tfMPTCanEscrow";
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTCanTrade"] = 16] = "tfMPTCanTrade";
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTCanTransfer"] = 32] = "tfMPTCanTransfer";
    MPTokenIssuanceCreateFlags[MPTokenIssuanceCreateFlags["tfMPTCanClawback"] = 64] = "tfMPTCanClawback";
})(MPTokenIssuanceCreateFlags || (exports.MPTokenIssuanceCreateFlags = MPTokenIssuanceCreateFlags = {}));
function validateMPTokenIssuanceCreate(tx) {
    var _a, _b;
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateOptionalField)(tx, 'MaximumAmount', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'MPTokenMetadata', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'TransferFee', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'AssetScale', common_1.isNumber);
    if (typeof tx.MPTokenMetadata === 'string' &&
        (!(0, utils_1.isHex)(tx.MPTokenMetadata) ||
            tx.MPTokenMetadata.length / 2 > mptokenMetadata_1.MAX_MPT_META_BYTE_LENGTH)) {
        throw new errors_1.ValidationError(`MPTokenIssuanceCreate: MPTokenMetadata (hex format) must be non-empty and no more than ${mptokenMetadata_1.MAX_MPT_META_BYTE_LENGTH} bytes.`);
    }
    if (typeof tx.MaximumAmount === 'string') {
        if (!utils_1.INTEGER_SANITY_CHECK.exec(tx.MaximumAmount)) {
            throw new errors_1.ValidationError('MPTokenIssuanceCreate: Invalid MaximumAmount');
        }
        else if (BigInt(tx.MaximumAmount) > BigInt(MAX_AMT) ||
            BigInt(tx.MaximumAmount) < BigInt(`0`)) {
            throw new errors_1.ValidationError('MPTokenIssuanceCreate: MaximumAmount out of range');
        }
    }
    if (typeof tx.TransferFee === 'number') {
        const flags = ((_a = tx.Flags) !== null && _a !== void 0 ? _a : 0);
        const isTfMPTCanTransfer = typeof flags === 'number'
            ? (0, utils_1.isFlagEnabled)(flags, MPTokenIssuanceCreateFlags.tfMPTCanTransfer)
            : ((_b = flags.tfMPTCanTransfer) !== null && _b !== void 0 ? _b : false);
        if (tx.TransferFee < 0 || tx.TransferFee > MAX_TRANSFER_FEE) {
            throw new errors_1.ValidationError(`MPTokenIssuanceCreate: TransferFee must be between 0 and ${MAX_TRANSFER_FEE}`);
        }
        if (tx.TransferFee && !isTfMPTCanTransfer) {
            throw new errors_1.ValidationError('MPTokenIssuanceCreate: TransferFee cannot be provided without enabling tfMPTCanTransfer flag');
        }
    }
    if (tx.MPTokenMetadata != null) {
        const validationMessages = (0, mptokenMetadata_1.validateMPTokenMetadata)(tx.MPTokenMetadata);
        if (validationMessages.length > 0) {
            const message = [
                mptokenMetadata_1.MPT_META_WARNING_HEADER,
                ...validationMessages.map((msg) => `- ${msg}`),
            ].join('\n');
            console.warn(message);
        }
    }
}
exports.validateMPTokenIssuanceCreate = validateMPTokenIssuanceCreate;
//# sourceMappingURL=MPTokenIssuanceCreate.js.map