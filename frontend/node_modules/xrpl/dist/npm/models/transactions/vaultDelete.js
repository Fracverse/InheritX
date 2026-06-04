"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultDelete = void 0;
const common_1 = require("./common");
function validateVaultDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
}
exports.validateVaultDelete = validateVaultDelete;
//# sourceMappingURL=vaultDelete.js.map