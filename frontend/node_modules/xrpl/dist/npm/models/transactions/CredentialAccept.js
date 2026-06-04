"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateCredentialAccept = void 0;
const common_1 = require("./common");
function validateCredentialAccept(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Account', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Issuer', common_1.isString);
    (0, common_1.validateCredentialType)(tx);
}
exports.validateCredentialAccept = validateCredentialAccept;
//# sourceMappingURL=CredentialAccept.js.map