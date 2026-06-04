"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultClawback = void 0;
const common_1 = require("./common");
function validateVaultClawback(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Holder', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'Amount', common_1.isClawbackAmount);
}
exports.validateVaultClawback = validateVaultClawback;
//# sourceMappingURL=vaultClawback.js.map