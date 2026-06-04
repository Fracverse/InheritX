"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validatePermissionedDomainSet = void 0;
const common_1 = require("./common");
const MAX_ACCEPTED_CREDENTIALS = 10;
function validatePermissionedDomainSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateOptionalField)(tx, 'DomainID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'AcceptedCredentials', common_1.isArray);
    (0, common_1.validateCredentialsList)(tx.AcceptedCredentials, tx.TransactionType, false, MAX_ACCEPTED_CREDENTIALS);
}
exports.validatePermissionedDomainSet = validatePermissionedDomainSet;
//# sourceMappingURL=permissionedDomainSet.js.map