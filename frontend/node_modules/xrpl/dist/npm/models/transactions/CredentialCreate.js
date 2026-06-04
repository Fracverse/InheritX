"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateCredentialCreate = void 0;
const utils_1 = require("@xrplf/isomorphic/utils");
const errors_1 = require("../../errors");
const common_1 = require("./common");
const MAX_URI_LENGTH = 256;
function validateCredentialCreate(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Account', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Subject', common_1.isString);
    (0, common_1.validateCredentialType)(tx);
    (0, common_1.validateOptionalField)(tx, 'Expiration', common_1.isNumber);
    validateURI(tx.URI);
}
exports.validateCredentialCreate = validateCredentialCreate;
function validateURI(URI) {
    if (URI === undefined) {
        return;
    }
    if (typeof URI !== 'string') {
        throw new errors_1.ValidationError('CredentialCreate: invalid field URI');
    }
    if (URI.length === 0) {
        throw new errors_1.ValidationError('CredentialCreate: URI cannot be an empty string');
    }
    else if (URI.length > MAX_URI_LENGTH) {
        throw new errors_1.ValidationError(`CredentialCreate: URI length must be <= ${MAX_URI_LENGTH}`);
    }
    if (!utils_1.HEX_REGEX.test(URI)) {
        throw new errors_1.ValidationError('CredentialCreate: URI must be encoded in hex');
    }
}
//# sourceMappingURL=CredentialCreate.js.map