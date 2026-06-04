"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateAccountDelete = void 0;
const common_1 = require("./common");
function validateAccountDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Destination', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'DestinationTag', common_1.isNumber);
    (0, common_1.validateCredentialsList)(tx.CredentialIDs, tx.TransactionType, true, common_1.MAX_AUTHORIZED_CREDENTIALS);
}
exports.validateAccountDelete = validateAccountDelete;
//# sourceMappingURL=accountDelete.js.map