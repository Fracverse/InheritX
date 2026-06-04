"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultCreate = exports.VaultCreateFlags = exports.VaultWithdrawalPolicy = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const mptokenMetadata_1 = require("../utils/mptokenMetadata");
const common_1 = require("./common");
const MAX_SCALE = 18;
var VaultWithdrawalPolicy;
(function (VaultWithdrawalPolicy) {
    VaultWithdrawalPolicy[VaultWithdrawalPolicy["vaultStrategyFirstComeFirstServe"] = 1] = "vaultStrategyFirstComeFirstServe";
})(VaultWithdrawalPolicy || (exports.VaultWithdrawalPolicy = VaultWithdrawalPolicy = {}));
var VaultCreateFlags;
(function (VaultCreateFlags) {
    VaultCreateFlags[VaultCreateFlags["tfVaultPrivate"] = 65536] = "tfVaultPrivate";
    VaultCreateFlags[VaultCreateFlags["tfVaultShareNonTransferable"] = 131072] = "tfVaultShareNonTransferable";
})(VaultCreateFlags || (exports.VaultCreateFlags = VaultCreateFlags = {}));
function validateVaultCreate(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Asset', common_1.isCurrency);
    (0, common_1.validateOptionalField)(tx, 'Data', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'AssetsMaximum', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'MPTokenMetadata', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'WithdrawalPolicy', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'DomainID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Scale', common_1.isNumber);
    if (tx.Data !== undefined) {
        const dataHex = tx.Data;
        if (!(0, utils_1.isHex)(dataHex)) {
            throw new errors_1.ValidationError('VaultCreate: Data must be a valid hex string');
        }
        const dataByteLength = dataHex.length / 2;
        if (dataByteLength > common_1.VAULT_DATA_MAX_BYTE_LENGTH) {
            throw new errors_1.ValidationError(`VaultCreate: Data exceeds ${common_1.VAULT_DATA_MAX_BYTE_LENGTH} bytes (actual: ${dataByteLength})`);
        }
    }
    if (tx.MPTokenMetadata !== undefined) {
        const metaHex = tx.MPTokenMetadata;
        if (!(0, utils_1.isHex)(metaHex)) {
            throw new errors_1.ValidationError('VaultCreate: MPTokenMetadata must be a valid non-empty hex string');
        }
        const metaByteLength = metaHex.length / 2;
        if (metaByteLength > mptokenMetadata_1.MAX_MPT_META_BYTE_LENGTH) {
            throw new errors_1.ValidationError(`VaultCreate: MPTokenMetadata exceeds ${mptokenMetadata_1.MAX_MPT_META_BYTE_LENGTH} bytes (actual: ${metaByteLength})`);
        }
    }
    if (tx.DomainID !== undefined &&
        !(0, utils_1.hasFlag)(tx, VaultCreateFlags.tfVaultPrivate, 'tfVaultPrivate')) {
        throw new errors_1.ValidationError('VaultCreate: Cannot set DomainID unless tfVaultPrivate flag is set.');
    }
    const asset = tx.Asset;
    const isXRP = asset.currency === 'XRP';
    const isMPT = 'mpt_issuance_id' in asset;
    const isIOU = !isXRP && !isMPT;
    if (tx.Scale !== undefined) {
        if (isXRP || isMPT) {
            throw new errors_1.ValidationError('VaultCreate: Scale parameter must not be provided for XRP or MPT assets');
        }
        if (isIOU) {
            if (!Number.isInteger(tx.Scale) || tx.Scale < 0 || tx.Scale > MAX_SCALE) {
                throw new errors_1.ValidationError(`VaultCreate: Scale must be a number between 0 and ${MAX_SCALE} inclusive for IOU assets`);
            }
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
exports.validateVaultCreate = validateVaultCreate;
//# sourceMappingURL=vaultCreate.js.map