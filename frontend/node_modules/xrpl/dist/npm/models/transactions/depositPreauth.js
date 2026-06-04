"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateDepositPreauth = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateDepositPreauth(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    validateSingleAuthorizationFieldProvided(tx);
    if (tx.Authorize !== undefined) {
        if (typeof tx.Authorize !== 'string') {
            throw new errors_1.ValidationError('DepositPreauth: Authorize must be a string');
        }
        if (tx.Account === tx.Authorize) {
            throw new errors_1.ValidationError("DepositPreauth: Account can't preauthorize its own address");
        }
    }
    else if (tx.Unauthorize !== undefined) {
        if (typeof tx.Unauthorize !== 'string') {
            throw new errors_1.ValidationError('DepositPreauth: Unauthorize must be a string');
        }
        if (tx.Account === tx.Unauthorize) {
            throw new errors_1.ValidationError("DepositPreauth: Account can't unauthorize its own address");
        }
    }
    else if (tx.AuthorizeCredentials !== undefined) {
        (0, common_1.validateCredentialsList)(tx.AuthorizeCredentials, tx.TransactionType, false, common_1.MAX_AUTHORIZED_CREDENTIALS);
    }
    else if (tx.UnauthorizeCredentials !== undefined) {
        (0, common_1.validateCredentialsList)(tx.UnauthorizeCredentials, tx.TransactionType, false, common_1.MAX_AUTHORIZED_CREDENTIALS);
    }
}
exports.validateDepositPreauth = validateDepositPreauth;
function validateSingleAuthorizationFieldProvided(tx) {
    const fields = [
        'Authorize',
        'Unauthorize',
        'AuthorizeCredentials',
        'UnauthorizeCredentials',
    ];
    const countProvided = fields.filter((key) => tx[key] !== undefined).length;
    if (countProvided !== 1) {
        throw new errors_1.ValidationError('DepositPreauth: Requires exactly one field of the following: Authorize, Unauthorize, AuthorizeCredentials, UnauthorizeCredentials.');
    }
}
//# sourceMappingURL=depositPreauth.js.map