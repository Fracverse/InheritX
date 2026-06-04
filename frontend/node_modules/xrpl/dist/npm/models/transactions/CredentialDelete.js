"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateCredentialDelete = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateCredentialDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    if (!tx.Subject && !tx.Issuer) {
        throw new errors_1.ValidationError('CredentialDelete: either `Issuer` or `Subject` must be provided');
    }
    (0, common_1.validateRequiredField)(tx, 'Account', common_1.isString);
    (0, common_1.validateCredentialType)(tx);
    (0, common_1.validateOptionalField)(tx, 'Subject', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Issuer', common_1.isString);
}
exports.validateCredentialDelete = validateCredentialDelete;
//# sourceMappingURL=CredentialDelete.js.map