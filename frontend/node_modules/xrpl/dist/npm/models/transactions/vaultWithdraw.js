"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateVaultWithdraw = void 0;
const common_1 = require("./common");
function validateVaultWithdraw(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Amount', common_1.isAmount);
    (0, common_1.validateOptionalField)(tx, 'Destination', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'DestinationTag', common_1.isNumber);
}
exports.validateVaultWithdraw = validateVaultWithdraw;
//# sourceMappingURL=vaultWithdraw.js.map