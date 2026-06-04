"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultSet = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
function validateVaultSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Data', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'AssetsMaximum', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'DomainID', common_1.isString);
    if (tx.Data !== undefined) {
        const dataHex = tx.Data;
        if (!(0, utils_1.isHex)(dataHex)) {
            throw new errors_1.ValidationError('VaultSet: Data must be a valid hex string');
        }
        const dataByteLength = dataHex.length / 2;
        if (dataByteLength > common_1.VAULT_DATA_MAX_BYTE_LENGTH) {
            throw new errors_1.ValidationError(`VaultSet: Data exceeds ${common_1.VAULT_DATA_MAX_BYTE_LENGTH} bytes (actual: ${dataByteLength})`);
        }
    }
}
exports.validateVaultSet = validateVaultSet;
//# sourceMappingURL=vaultSet.js.map