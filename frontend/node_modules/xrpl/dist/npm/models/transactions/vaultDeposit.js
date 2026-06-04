"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultDeposit = void 0;
const common_1 = require("./common");
function validateVaultDeposit(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Amount', common_1.isAmount);
}
exports.validateVaultDeposit = validateVaultDeposit;
//# sourceMappingURL=vaultDeposit.js.map