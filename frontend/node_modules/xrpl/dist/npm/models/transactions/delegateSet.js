"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateDelegateSet = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
const PERMISSIONS_MAX_LENGTH = 10;
const NON_DELEGABLE_TRANSACTIONS = new Set([
    'AccountSet',
    'SetRegularKey',
    'SignerListSet',
    'DelegateSet',
    'AccountDelete',
    'Batch',
    'EnableAmendment',
    'SetFee',
    'UNLModify',
]);
function validateDelegateSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'Authorize', common_1.isAccount);
    if (tx.Authorize === tx.Account) {
        throw new errors_1.ValidationError('DelegateSet: Authorize and Account must be different.');
    }
    (0, common_1.validateRequiredField)(tx, 'Permissions', Array.isArray);
    const permissions = tx.Permissions;
    if (permissions.length > PERMISSIONS_MAX_LENGTH) {
        throw new errors_1.ValidationError(`DelegateSet: Permissions array length cannot be greater than ${PERMISSIONS_MAX_LENGTH}.`);
    }
    const permissionValueSet = new Set();
    permissions.forEach((permission) => {
        if (permission == null ||
            Object.keys(permission).length !== 1 ||
            permission.Permission == null ||
            Object.keys(permission.Permission).length !== 1) {
            throw new errors_1.ValidationError('DelegateSet: Permissions array element is malformed');
        }
        const permissionValue = permission.Permission.PermissionValue;
        if (permissionValue == null) {
            throw new errors_1.ValidationError('DelegateSet: PermissionValue must be defined');
        }
        if (typeof permissionValue !== 'string') {
            throw new errors_1.ValidationError('DelegateSet: PermissionValue must be a string');
        }
        if (NON_DELEGABLE_TRANSACTIONS.has(permissionValue)) {
            throw new errors_1.ValidationError(`DelegateSet: PermissionValue contains a non-delegatable transaction ${permissionValue}`);
        }
        permissionValueSet.add(permissionValue);
    });
    if (permissions.length !== permissionValueSet.size) {
        throw new errors_1.ValidationError('DelegateSet: Permissions array cannot contain duplicate values');
    }
}
exports.validateDelegateSet = validateDelegateSet;
//# sourceMappingURL=delegateSet.js.map